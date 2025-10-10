use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField};
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
            Action::SetChild(TypeField::NoteOn(note_on)) => {
                let prev = self.note_on;
                self.note_on = *note_on;
                Action::SetChild(TypeField::NoteOn(prev))
            }
            Action::SetChild(TypeField::NoteDeleted(note_deleted)) => {
                let prev = self.note_deleted;
                self.note_deleted = *note_deleted;
                Action::SetChild(TypeField::NoteDeleted(prev))
            }
            _ => return None,
        })
    }
}
