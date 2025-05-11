use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use shared::model::SimpleWaveConfig;

impl ActionReceiver for SimpleWaveConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::Wave(wave)) => {
                let prev = self.wave;
                self.wave = *wave;
                Action::SetChild(TypeField::Wave(prev))
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
            Action::SetChild(TypeField::Envelope(envelope)) => {
                let prev = self.envelope.clone();
                self.envelope = envelope.clone();
                Action::SetChild(TypeField::Envelope(prev))
            }
            Action::SetChild(TypeField::AntiAliasingMode(mode)) => {
                let prev = self.anti_aliasing_mode;
                self.anti_aliasing_mode = *mode;
                Action::SetChild(TypeField::AntiAliasingMode(prev))
            }
            Action::SetUint(UintField::OversampleFactor, factor) => {
                let prev = self.oversample_factor;
                self.oversample_factor = *factor;
                Action::SetUint(UintField::OversampleFactor, prev)
            }
            Action::SetChild(TypeField::PolyphonyMode(mode)) => {
                let prev = self.polyphony_mode;
                self.polyphony_mode = *mode;
                Action::SetChild(TypeField::PolyphonyMode(prev))
            }
            Action::SetUint(UintField::PolyphonyLimit, factor) => {
                let prev = self.polyphony_limit;
                self.polyphony_limit = *factor;
                Action::SetUint(UintField::PolyphonyLimit, prev)
            }
            _ => return None,
        })
    }
}
