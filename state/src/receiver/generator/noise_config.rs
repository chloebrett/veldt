use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::NoiseConfig;

impl ActionReceiver for NoiseConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::NoiseType(noise)) => {
                let prev = self.kind;
                self.kind = *noise;
                Action::SetChild(TypeField::NoiseType(prev))
            }
            _ => return None,
        })
    }
}
