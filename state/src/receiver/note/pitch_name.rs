use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::PitchName;

impl ActionReceiver for PitchName {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetScaleValue(new_note) => {
                let prev = self.scale_value;
                self.scale_value = *new_note;
                Action::SetScaleValue(prev)
            }
            Action::SetOctave(octave) => {
                let prev = self.octave;
                self.octave = *octave;
                Action::SetOctave(prev)
            }
            _ => return None,
        })
    }
}
