use crate::consts::{MS_PER_SECOND, SAMPLE_RATE};
use shared::model::AdsrEnvelope;
use shared::types::Milliseconds;

/// Applies an ADSR envelope given a particular index within a set of samples.
/// Returns the corresponding amplitude multiplier.
/// E.g. sample_index = 0 would be the first sample, and would therefore return the amplitude
/// corresponding to the start of the envelope (usually zero, unless attack = 0 and there is
/// sustain/decay).
/// Note: this envelope doesn't correctly handle several things.
/// For example, release is treated as part of the main envelope.
pub fn trivial_envelope(sample_index: i32, envelope: &AdsrEnvelope, duration: Milliseconds) -> f32 {
    // TODO: Treat the envelope as a state machine for each synth voice.
    // The release should not be part of the envelope duration.
    let ms = sample_index as f32 / (SAMPLE_RATE as f32) * MS_PER_SECOND;

    // TODO: apply exponential curves to the envelope.
    if ms < envelope.attack {
        // in attack
        ms / envelope.attack
    } else if ms < envelope.attack + envelope.decay {
        // in decay
        (envelope.attack - ms) / envelope.decay * (1.0 - envelope.sustain) + 1.0
    } else if ms < duration - envelope.release {
        // in sustain
        envelope.sustain
    } else {
        // in release
        (duration - ms) / envelope.release * envelope.sustain
    }
}

#[derive(Debug)]
enum EnvelopeState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
    _Shutdown,
}

#[derive(Debug)]
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

    fn check_state_transitions(&mut self) {
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
        self.check_state_transitions();
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // TODO: make the errors for this more readable,
        // and perhaps make our own macro.
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
}
