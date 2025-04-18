use crate::Action;
use crate::receiver::ActionReceiver;
use ordered_float::OrderedFloat;
use shared::model::PlacedNote;

impl ActionReceiver for PlacedNote {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetNoteOffset(offset) => {
                let prev = self.offset;
                self.offset = OrderedFloat(*offset);
                Action::SetNoteOffset(*prev)
            }
            _ => return self.note.apply(action),
        })
    }
}
