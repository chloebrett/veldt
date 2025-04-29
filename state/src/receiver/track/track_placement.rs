use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use ordered_float::OrderedFloat;
use shared::model::TrackPlacement;

impl ActionReceiver for TrackPlacement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetUint(UintField::TrackId, track_id) => {
                let prev = self.track_id;
                self.track_id = *track_id;
                Action::SetUint(UintField::TrackId, prev)
            }
            Action::SetFloat(FloatField::Offset, offset) => {
                let prev = self.offset;
                self.offset = OrderedFloat(*offset);
                Action::SetFloat(FloatField::Offset, *prev)
            }
            Action::SetChild(TypeField::ClippedDuration(duration)) => {
                let prev = self.clipped_duration.as_deref().copied();
                self.clipped_duration = duration.map(OrderedFloat);
                Action::SetChild(TypeField::ClippedDuration(prev))
            }
            Action::SetUint(UintField::GeneratorIndex, generator_index) => {
                let prev = self.generator_index;
                self.generator_index = *generator_index as usize;
                Action::SetUint(UintField::GeneratorIndex, prev as u32)
            }
            _ => return None,
        })
    }
}
