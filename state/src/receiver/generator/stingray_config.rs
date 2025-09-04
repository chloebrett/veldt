use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::StingrayConfig;

impl ActionReceiver for StingrayConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::LpfOn(on)) => {
                let prev = self.lpf_on;
                self.lpf_on = *on;
                Action::SetChild(TypeField::LpfOn(prev))
            }
            _ => return None,
        })
    }
}
