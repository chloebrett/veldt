use crate::Action;
use crate::receiver::ActionReceiver;
use ordered_float::OrderedFloat;
use shared::model::Track;

impl ActionReceiver for Track {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::DeleteNote(note_index) => {
                let prev = self
                    .notes
                    .get(*note_index)
                    .expect("Can't delete non-existent track!")
                    .clone();
                self.notes.remove(*note_index);
                Action::AddNote(prev)
            }
            Action::AddNote(note) => {
                let index = self.notes.len();
                self.notes.push(note.clone());
                Action::DeleteNote(index)
            }
            Action::SetTrackOffset(offset) => {
                let prev = self.offset;
                self.offset = OrderedFloat(*offset);
                Action::SetTrackOffset(*prev)
            }
            _ => return None,
        })
    }
}
