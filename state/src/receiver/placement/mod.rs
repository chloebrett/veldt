mod sample_placement;
mod track_placement;

use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use ordered_float::OrderedFloat;
use shared::model::{Placement, PlacementType};

impl ActionReceiver for Placement {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = match &mut self.kind {
            PlacementType::Track(track_placement) => track_placement.apply(action),
            PlacementType::Sample(sample_placement) => sample_placement.apply(action),
        } {
            return Some(undo);
        }

        Some(match action {
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
            Action::SetUint(UintField::VisualPlacement, position) => {
                let prev = self.visual_placement;
                self.visual_placement = *position;
                Action::SetUint(UintField::VisualPlacement, prev)
            }
            _ => return None,
        })
    }
}
