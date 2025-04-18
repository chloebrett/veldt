use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use ordered_float::OrderedFloat;
use shared::model::PlacedNote;

impl ActionReceiver for PlacedNote {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.note.apply(action) {
            return Some(undo);
        }

        Some(match action {
            Action::SetFloat(FloatField::Offset, offset) => {
                let prev = self.offset;
                self.offset = OrderedFloat(*offset);
                Action::SetFloat(FloatField::Offset, *prev)
            }
            _ => return None,
        })
    }
}
