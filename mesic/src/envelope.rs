use crate::consts::{MS_PER_SECOND, SAMPLE_RATE};
use shared::model::{AdsrEnvelope, ModMatrix};

#[derive(Debug, Clone)]
enum EnvelopeState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
    _Shutdown,
}

#[derive(Debug, Clone)]
pub struct EnvelopeGenerator {
    state: EnvelopeState,
    config: AdsrEnvelope,
    last_output: f32,
    attack_per_sample: f32,
    decay_per_sample: f32,
    release_per_sample: f32,
}

impl EnvelopeGenerator {
    pub fn new(envelope: AdsrEnvelope) -> Self {
        let attack_per_sample = MS_PER_SECOND / (envelope.attack * SAMPLE_RATE as f32);
        let decay_per_sample = MS_PER_SECOND / (envelope.decay * SAMPLE_RATE as f32);
        let release_per_sample = MS_PER_SECOND / (envelope.release * SAMPLE_RATE as f32);

        Self {
            state: EnvelopeState::Off,
            config: envelope,
            last_output: 0.0,
            attack_per_sample,
            decay_per_sample,
            release_per_sample,
        }
    }

    pub fn note_on(&mut self) {
        match self.state {
            EnvelopeState::Off => {
                if self.config.attack > 0.0 {
                    self.state = EnvelopeState::Attack;
                } else if self.config.decay > 0.0 {
                    self.state = EnvelopeState::Decay;
                } else {
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Release => {
                self.state = EnvelopeState::Attack;
            }
            _ => {}
        }
    }

    pub fn note_off(&mut self) {
        match self.state {
            EnvelopeState::Attack | EnvelopeState::Decay | EnvelopeState::Sustain => {
                self.state = EnvelopeState::Release
            }
            _ => {}
        }
    }

    fn _shutdown(&mut self) {
        match self.state {
            EnvelopeState::Attack | EnvelopeState::Decay | EnvelopeState::Sustain => {
                self.state = EnvelopeState::_Shutdown
            }
            _ => {}
        }
    }

    /// Update state if the output triggers a state change.
    fn maybe_update_state(&mut self) {
        let output = self.last_output;
        match self.state {
            EnvelopeState::Attack if output >= 1.0 => {
                self.state = EnvelopeState::Decay;
            }
            EnvelopeState::Decay if output <= self.config.sustain => {
                self.state = EnvelopeState::Sustain;
            }
            EnvelopeState::Release | EnvelopeState::_Shutdown if output <= 0.0 => {
                self.state = EnvelopeState::Off;
            }
            _ => {}
        }
    }

    pub fn set_envelope(&mut self, envelope: AdsrEnvelope) {
        self.config = envelope;
    }

    pub fn update_envelope(
        &mut self,
        osc_index: usize,
        envelopes: &[AdsrEnvelope],
        matrix: &ModMatrix,
    ) {
        let mut new_env = AdsrEnvelope {
            attack: 0.0,
            decay: 0.0,
            sustain: 0.0,
            release: 0.0,
        };

        for (i, env) in envelopes.iter().enumerate() {
            let weight = matrix.get(i, osc_index).map_or(0.0, |c| (*c).into());

            if weight != 0.0 {
                new_env.attack += weight * env.attack;
                new_env.decay += weight * env.decay;
                new_env.sustain += weight * env.sustain;
                new_env.release += weight * env.release;
            }
        }

        new_env.attack = new_env.attack.clamp(0.0, 1000.0);
        new_env.decay = new_env.decay.clamp(0.0, 1000.0);
        new_env.sustain = new_env.sustain.clamp(0.0, 1.0);
        new_env.release = new_env.release.clamp(0.0, 1000.0);

        self.config = new_env;
    }
}

const SHUTDOWN_MS: f32 = 1.0;
const SHUTDOWN_PER_SAMPLE: f32 = MS_PER_SECOND / (SHUTDOWN_MS * SAMPLE_RATE as f32);

impl Iterator for EnvelopeGenerator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let output: f32 = match self.state {
            EnvelopeState::Off => 0.0,
            EnvelopeState::Attack => self.last_output + self.attack_per_sample,
            EnvelopeState::Decay => self.last_output - self.decay_per_sample,
            EnvelopeState::Sustain => self.config.sustain,
            EnvelopeState::Release => self.last_output - self.release_per_sample,
            EnvelopeState::_Shutdown => self.last_output - SHUTDOWN_PER_SAMPLE,
        };

        let output = output.clamp(0.0, 1.0);
        self.last_output = output;
        self.maybe_update_state();
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::model::ModMatrix;

    const FLOAT_THRES: f32 = 1e-6;

    #[test]
    fn eg_off() {
        let envelope = AdsrEnvelope {
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
        };
        let eg = EnvelopeGenerator::new(envelope);

        let result: Vec<_> = eg.take(100).collect();
        assert_eq!(result, vec![0.0; 100]);
    }

    #[test]
    fn eg_on_sustain_only() {
        let envelope = AdsrEnvelope {
            attack: 0.0,
            decay: 0.0,
            sustain: 0.9,
            release: 0.0,
        };
        let mut eg = EnvelopeGenerator::new(envelope);
        eg.note_on();

        let result: Vec<_> = eg.take(100).collect();
        assert_eq!(result, vec![0.9; 100]);
    }

    #[test]
    fn eg_on_adsr() {
        let five_samples_in_ms = 5.0 / SAMPLE_RATE as f32 * 1000.0;
        let envelope = AdsrEnvelope {
            attack: five_samples_in_ms,
            decay: five_samples_in_ms,
            sustain: 0.5,
            release: five_samples_in_ms,
        };
        let mut eg = EnvelopeGenerator::new(envelope);
        eg.note_on();

        let mut result: Vec<_> = eg.by_ref().take(20).collect();
        eg.note_off();
        result.extend(eg.take(10));

        let mut expected = vec![];
        // Attack
        expected.extend([0.2, 0.4, 0.6, 0.8, 1.0]);
        // Decay
        expected.extend([0.8, 0.6, 0.4]);
        // Sustain
        expected.extend([0.5; 12]);
        // Release
        expected.extend([0.3, 0.1]);
        // Off
        expected.extend([0.0; 8]);
        assert_almost_equal(result, expected);
    }

    fn assert_almost_equal(first: Vec<f32>, second: Vec<f32>) {
        if first.len() != second.len() {
            panic!("Lengths differed! {}, {}", first.len(), second.len());
        }
        for i in 0..first.len() {
            let a = first[i];
            let b = second[i];
            if (a - b).abs() > FLOAT_THRES {
                panic!("Floats {a}, {b} differed at index {i}");
            }
        }
    }

    #[test]
    fn test_additive_envelope() {
        let envelopes = vec![
            AdsrEnvelope {
                attack: 1000.0,
                decay: 200.0,
                sustain: 1.0,
                release: 1000.0,
            },
            AdsrEnvelope {
                attack: 0.0,
                decay: 1000.0,
                sustain: 0.5,
                release: 0.0,
            },
        ];

        let mut eg = EnvelopeGenerator {
            state: EnvelopeState::Attack,
            config: AdsrEnvelope {
                attack: 0.0,
                decay: 0.0,
                sustain: 0.0,
                release: 0.0,
            },
            last_output: 0.0,
            attack_per_sample: 0.0,
            decay_per_sample: 0.0,
            release_per_sample: 0.0,
        };

        let a = 0.7;
        let b = 0.3;

        let mut mod_matrix = ModMatrix::new(6, 4);
        mod_matrix.get_mut(0, 0).unwrap().set(a);
        mod_matrix.get_mut(1, 0).unwrap().set(b);

        eg.update_envelope(0, &envelopes, &mod_matrix);

        let expected_attack =
            (a * envelopes[0].attack + b * envelopes[1].attack).clamp(0.0, 1000.0);
        let expected_decay = (a * envelopes[0].decay + b * envelopes[1].decay).clamp(0.0, 1000.0);
        let expected_sustain =
            (a * envelopes[0].sustain + b * envelopes[1].sustain).clamp(0.0, 1.0);
        let expected_release =
            (a * envelopes[0].release + b * envelopes[1].release).clamp(0.0, 1000.0);

        let env = eg.config;
        assert!((env.attack - expected_attack).abs() < 1e-6);
        assert!((env.decay - expected_decay).abs() < 1e-6);
        assert!((env.sustain - expected_sustain).abs() < 1e-6);
        assert!((env.release - expected_release).abs() < 1e-6);
    }
}
