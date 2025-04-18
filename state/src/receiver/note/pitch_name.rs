use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::PitchName;

impl ActionReceiver for PitchName {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetNoteScaleValue(new_note) => {
                let prev = self.scale_value;
                self.scale_value = *new_note;
                Action::SetNoteScaleValue(prev)
            }
            Action::SetNoteOctave(octave) => {
                let prev = self.octave;
                self.octave = *octave;
                Action::SetNoteOctave(prev)
            }
            _ => return None,
        })
    }
}
