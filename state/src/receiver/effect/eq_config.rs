use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField};
use shared::model::EqConfig;

impl ActionReceiver for EqConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::EqType(kind)) => {
                let prev = self.kind.clone();
                self.kind = kind.clone();
                Action::SetChild(TypeField::EqType(prev))
            }
            Action::SetFloat(FloatField::Fc, fc) => {
                let prev = self.fc;
                self.fc = *fc;
                Action::SetFloat(FloatField::Fc, prev)
            }
            Action::SetFloat(FloatField::Q, q) => {
                let prev = self.q;
                self.q = *q;
                Action::SetFloat(FloatField::Q, prev)
            }
            Action::SetFloat(FloatField::Gain, gain) => {
                let prev = self.gain;
                self.gain = *gain;
                Action::SetFloat(FloatField::Gain, prev)
            }
            _ => return None,
        })
    }
}
