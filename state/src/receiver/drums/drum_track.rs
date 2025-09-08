use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::{DrumTrack};

impl ActionReceiver for DrumTrack {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddChild(TypeField::SampleId(sample_id)) => {
                // TODO add to drum sub tracks, use sample_id as key
                Action::DeleteChildById(TypeField::SampleId(*sample_id))
            }
            // TODO delete sub track
            _ => return None,
        })
    }
}
