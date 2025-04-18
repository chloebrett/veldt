use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::Note;

impl ActionReceiver for Note {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.pitch_name.apply(action) {
            return Some(undo);
        }

        Some(match action {
            Action::SetNotePitchName(pitch_name) => {
                let prev = self.pitch_name;
                self.pitch_name = *pitch_name;
                Action::SetNotePitchName(prev)
            }
            Action::SetFloat(FloatField::Duration, duration) => {
                let prev = self.beats;
                self.beats = *duration;
                Action::SetFloat(FloatField::Duration, prev)
            }
            _ => return None,
        })
    }
}
