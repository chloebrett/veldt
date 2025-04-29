use crate::receiver::ActionReceiver;
use crate::{Action, IndexField};
use shared::model::SamplePlacement;

impl ActionReceiver for SamplePlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetIndex(IndexField::Sample(sample_index)) => {
                let prev = self.sample_index;
                self.sample_index = *sample_index;
                Action::SetIndex(IndexField::Sample(prev))
            }
            _ => return None,
        })
    }
}
