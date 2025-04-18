use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, UintField};
use shared::model::SimpleWaveConfig;

impl ActionReceiver for SimpleWaveConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetWave(wave) => {
                let prev = self.wave;
                self.wave = *wave;
                Action::SetWave(prev)
            }
            Action::SetUint(UintField::OscCount, osc_count) => {
                let prev = self.osc_count;
                self.osc_count = *osc_count;
                Action::SetUint(UintField::OscCount, prev)
            }
            Action::SetFloat(FloatField::Detune, detune_cents) => {
                let prev = self.detune_cents;
                self.detune_cents = *detune_cents;
                Action::SetFloat(FloatField::Detune, prev)
            }
            Action::SetEnvelope(envelope) => {
                let mut envelope = envelope.clone();
                let headroom = 1.0 - envelope.attack - envelope.decay - envelope.release;
                let max_attack = headroom + envelope.attack;
                let max_decay = headroom + envelope.decay;
                let max_release = headroom + envelope.release;

                if envelope.attack > max_attack {
                    envelope.attack = max_attack;
                }
                if envelope.decay > max_decay {
                    envelope.decay = max_decay;
                }
                if envelope.release > max_release {
                    envelope.release = max_release;
                }

                let prev = self.envelope.clone();
                self.envelope = envelope;
                Action::SetEnvelope(prev)
            }
            Action::SetAntiAliasingMode(mode) => {
                let prev = self.anti_aliasing_mode;
                self.anti_aliasing_mode = *mode;
                Action::SetAntiAliasingMode(prev)
            }
            Action::SetUint(UintField::OversampleFactor, factor) => {
                let prev = self.oversample_factor;
                self.oversample_factor = *factor;
                Action::SetUint(UintField::OversampleFactor, prev)
            }
            _ => return None,
        })
    }
}
