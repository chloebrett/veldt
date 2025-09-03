use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::DrumPlacement;

impl ActionReceiver for DrumPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::DrumTrackId(drum_track_id)) => {
                let prev = self.drum_track_id;
                self.drum_track_id = *drum_track_id;
                Action::SetChild(TypeField::DrumTrackId(prev))
            }
            _ => return None,
        })
    }
}
