use crate::receiver::ActionReceiver;
use crate::{Action, TypeField};
use shared::model::PitchName;

impl ActionReceiver for PitchName {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetChild(TypeField::ScaleValue(new_note)) => {
                let prev = self.scale_value;
                self.scale_value = *new_note;
                Action::SetChild(TypeField::ScaleValue(prev))
            }
            Action::SetChild(TypeField::Octave(octave)) => {
                let prev = self.octave;
                self.octave = *octave;
                Action::SetChild(TypeField::Octave(prev))
            }
            _ => return None,
        })
    }
}
