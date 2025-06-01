use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MultiIndexField, MultiTypeField, TypeField};
use ordered_float::OrderedFloat;
use shared::model::Track;

use super::delete_elems;

impl ActionReceiver for Track {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::DeleteChild(IndexField::PlacedNote(note_index)) => {
                let prev = self
                    .notes
                    .get(*note_index)
                    .expect("Can't delete non-existent track!")
                    .clone();
                self.notes.remove(*note_index);
                Action::AddChild(TypeField::PlacedNote(prev))
            }
            Action::AddChild(TypeField::PlacedNote(note)) => {
                let index = self.notes.len();
                self.notes.push(note.clone());
                Action::DeleteChild(IndexField::PlacedNote(index))
            }
            Action::SetFloat(FloatField::Offset, offset) => {
                let prev = self.offset;
                self.offset = OrderedFloat(*offset);
                Action::SetFloat(FloatField::Offset, *prev)
            }
            Action::DeleteChildren(MultiIndexField::PlacedNote(indexes)) => {
                let prev = self.notes.clone();
                delete_elems(&mut self.notes, indexes.clone());
                Action::SetChildren(MultiTypeField::PlacedNote(prev))
            }
            Action::SetChildren(MultiTypeField::PlacedNote(notes)) => {
                let prev = self.notes.clone();
                self.notes = notes.to_vec();
                Action::SetChildren(MultiTypeField::PlacedNote(prev))
            }
            Action::AddChildren(MultiTypeField::PlacedNote(notes)) => {
                let prev = self.notes.clone();
                self.notes.extend(notes.to_vec());
                Action::SetChildren(MultiTypeField::PlacedNote(prev))
            }
            _ => return None,
        })
    }
}
