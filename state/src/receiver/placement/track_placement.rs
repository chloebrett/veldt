use crate::receiver::ActionReceiver;
use crate::{Action, IndexField, TypeField};
use shared::model::TrackPlacement;

impl ActionReceiver for TrackPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::TrackId(track_id)) => {
                let prev = self.track_id;
                self.track_id = *track_id;
                Action::SetChild(TypeField::TrackId(prev))
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
