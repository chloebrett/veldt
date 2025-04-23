use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use shared::model::OscillatorConfig;

impl ActionReceiver for OscillatorConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.apply(action) {
            return Some(undo);
        }
        Some(match action {
            Action::SetChild(TypeField::Wave(wave)) => {
                let prev = self.wave;
                self.wave = *wave;
                Action::SetChild(TypeField::Wave(prev))
            }
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetFloat(FloatField::Pan, pan) => {
                let prev = self.pan;
                self.pan = *pan;
                Action::SetFloat(FloatField::Pan, prev)
            }
            Action::SetFloat(FloatField::Detune, detune) => {
                let prev = self.osc_detune;
                self.osc_detune = *detune;
                Action::SetFloat(FloatField::Detune, prev)
            }
            Action::SetUint(UintField::OscCount, osc_count) => {
                let prev = self.osc_count;
                self.osc_count = *osc_count;
                Action::SetUint(UintField::OscCount, prev)
            }
            _ => return None,
        })
    }
}
