use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::{DrumSubTrack, DrumTrack};

impl ActionReceiver for DrumTrack {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddChild(TypeField::SampleId(sample_id)) => {
                self.drum_sub_tracks.insert(*sample_id, DrumSubTrack::default());
                Action::DeleteChildById(TypeField::SampleId(*sample_id))
            }
            Action::DeleteChildById(TypeField::SampleId(sample_id)) => {
                self.drum_sub_tracks.remove(sample_id);
                Action::AddChild(TypeField::SampleId(*sample_id))
            }
            _ => return None,
        })
    }
}
