use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::SamplePlacement;

impl ActionReceiver for SamplePlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::SampleId(sample_id)) => {
                let prev = self.sample_id;
                self.sample_id = *sample_id;
                Action::SetChild(TypeField::SampleId(prev))
            }
            _ => return None,
        })
    }
}
