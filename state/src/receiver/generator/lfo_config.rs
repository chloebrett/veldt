use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField};
use shared::model::LfoConfig;

impl ActionReceiver for LfoConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::Wave(wave)) => {
                let prev = self.wave;
                self.wave = *wave;
                Action::SetChild(TypeField::Wave(prev))
            }
            Action::SetFloat(FloatField::LfoFreq, frequency) => {
                let prev = self.frequency;
                self.frequency = *frequency;
                Action::SetFloat(FloatField::LfoFreq, prev)
            }
            _ => return None,
        })
    }
}
