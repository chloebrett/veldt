use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use shared::model::OscillatorConfig;

impl ActionReceiver for OscillatorConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::Wave(wave)) => {
                let prev = self.wave;
                self.wave = *wave;
                Action::SetChild(TypeField::Wave(prev))
            }
            Action::SetFloat(FloatField::Volume, new_volume) => {
                let prev = self.volume;
                self.volume = *new_volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetFloat(FloatField::Pan, new_pan) => {
                let prev = self.pan;
                self.pan = *new_pan;
                Action::SetFloat(FloatField::Pan, prev)
            }
            Action::SetFloat(FloatField::Detune, new_detune) => {
                let prev = self.osc_detune;
                self.osc_detune = *new_detune;
                Action::SetFloat(FloatField::Detune, prev)
            }
            Action::SetUint(UintField::OscCount, osc_count) => {
                let prev = self.osc_count;
                self.osc_count = *osc_count as f32;
                Action::SetUint(UintField::OscCount, prev as u32)
            }
            Action::SetFloat(FloatField::Detune, new_detune) => {
                let prev = self.unison_detune;
                self.unison_detune = *new_detune;
                Action::SetFloat(FloatField::Detune, prev)
            }
            _ => return None,
        })
    }
}