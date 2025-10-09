use crate::receiver::ActionReceiver;
use crate::{Action, TypeField, IndexField};
use shared::model::DrumTrackPlacement;

impl ActionReceiver for DrumTrackPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::TrackId(track_id)) => {
                let prev = self.track_id;
                self.track_id = *track_id;
                Action::SetChild(TypeField::TrackId(prev))
            }
            Action::SetChildById(
                TypeField::PitchName(pitch_name),
                TypeField::SampleId(sample_id),
            ) => {
                let pitch_value: i32 = pitch_name.clone().into();
                self.pitch_sample_map.insert(pitch_value, *sample_id);
                if let Some(prev_sample_id) = self.pitch_sample_map.get(&pitch_value) {
                    Action::SetChildById(
                        TypeField::PitchName(*pitch_name),
                        TypeField::SampleId(*prev_sample_id),
                    )
                } else {
                    Action::DeleteChildById(TypeField::PitchName(*pitch_name))
                }
            }
            Action::DeleteChildById(TypeField::PitchName(pitch_name)) => {
                let pitch_value: i32 = pitch_name.clone().into();
                // This action is only used as an undo function for the action above, so should be safe to directly unwrap in the line below
                let sample_id = self.pitch_sample_map.remove(&pitch_value).unwrap();
                Action::SetChildById(
                    TypeField::PitchName(*pitch_name),
                    TypeField::SampleId(sample_id),
                )
            }
            Action::SetIndex(IndexField::Mixer(mixer_channel)) => {
                let prev = self.mixer_channel;
                self.mixer_channel = *mixer_channel;
                Action::SetIndex(IndexField::Mixer(prev))
            }
            _ => return None,
        })
    }
}
