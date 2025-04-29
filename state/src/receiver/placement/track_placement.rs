use crate::receiver::ActionReceiver;
use crate::{Action, IndexField};
use shared::model::TrackPlacement;

impl ActionReceiver for TrackPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetIndex(IndexField::Track(track_index)) => {
                let prev = self.track_index;
                self.track_index = *track_index;
                Action::SetIndex(IndexField::Track(prev))
            }
            Action::SetIndex(IndexField::Generator(generator_index)) => {
                let prev = self.generator_index;
                self.generator_index = *generator_index;
                Action::SetIndex(IndexField::Generator(prev))
            }
            _ => return None,
        })
    }
}
