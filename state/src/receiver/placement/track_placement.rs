use crate::receiver::ActionReceiver;
use crate::{Action, IndexField, TypeField};
use shared::model::{GeneratorId, TrackPlacement};

impl ActionReceiver for TrackPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetIndex(IndexField::Track(track_index)) => {
                let prev = self.track_index;
                self.track_index = *track_index;
                Action::SetIndex(IndexField::Track(prev))
            }
            Action::SetChild(TypeField::GeneratorId(generator_id)) => {
                let prev = self.generator_id;
                self.generator_id = *generator_id;
                Action::SetChild(TypeField::GeneratorId(prev))
            }
            _ => return None,
        })
    }
}
