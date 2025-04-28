mod track_placement;

use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MultiIndexField, MultiTypeField, TypeField};
use ordered_float::OrderedFloat;
use shared::model::Track;

impl ActionReceiver for Track {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::DeleteChild(IndexField::PlacedNote(note_index)) => {
                let prev = self
                    .notes
                    .get(*note_index)
                    .expect("Can't delete non-existent note!")
                    .clone();
                self.notes.remove(*note_index);
                Action::AddChild(TypeField::PlacedNote(prev))
            }
            Action::DeleteChildren(MultiIndexField::PlacedNote(note_index_tree)) => {
                let prev = self
                    .notes
                    .iter()
                    .map(|note| TypeField::PlacedNote(note.clone()))
                    .collect();
                // Iterate in reverse so that removal note indexes do not change duration loop.
                for index in note_index_tree.into_iter().rev() {
                    self.notes.remove(*index);
                }
                Action::SetChildren(MultiTypeField { values: prev })
            }
            Action::AddChild(TypeField::PlacedNote(note)) => {
                let index = self.notes.len();
                self.notes.push(note.clone());
                Action::DeleteChild(IndexField::PlacedNote(index))
            }
            Action::SetChildren(MultiTypeField { values }) => {
                let prev = self
                    .notes
                    .iter()
                    .map(|note| TypeField::PlacedNote(note.clone()))
                    .collect();
                self.notes = vec![];
                for type_field in values {
                    if let TypeField::PlacedNote(note) = type_field {
                        self.notes.push(note.clone())
                    }
                }
                Action::SetChildren(MultiTypeField { values: prev })
            }
            Action::SetFloat(FloatField::Offset, offset) => {
                let prev = self.offset;
                self.offset = OrderedFloat(*offset);
                Action::SetFloat(FloatField::Offset, *prev)
            }
            _ => return None,
        })
    }
}
