use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::DrumPlacement;

//TODO: double check grouping of placements
impl ActionReceiver for DrumPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::TrackId(track_id)) => {
                let prev = self.track_id;
                self.track_id = *track_id;
                Action::SetChild(TypeField::TrackId(prev))
            }
            _ => return None,
        })
    }
}
