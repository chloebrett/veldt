use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::DrumTrackPlacement;

impl ActionReceiver for DrumTrackPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::TrackId(track_id)) => {
                let prev = self.track_id;
                self.track_id = *track_id;
                Action::SetChild(TypeField::TrackId(prev))
            }
            Action::SetChild(TypeField::SampleId(sample_id)) => {
                let prev = self.sample_id;
                self.sample_id = *sample_id;
                Action::SetChild(TypeField::SampleId(prev))
            }
            _ => return None,
        })
    }
}
