use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::StingrayConfig;

impl ActionReceiver for StingrayConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::Mute(bool)) => {
                let prev = self.lpf_enabled;
                self.lpf_enabled = *bool;
                Action::SetChild(TypeField::Mute(prev))
            }
            _ => return None,
        })
    }
}
