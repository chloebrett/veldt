use crate::Action;
use crate::receiver::ActionReceiver;
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
            Action::SetNoteDuration(duration) => {
                let prev = self.beats;
                self.beats = *duration;
                Action::SetNoteDuration(prev)
            }
            _ => return None,
        })
    }
}
