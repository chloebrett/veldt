use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::Note;

impl ActionReceiver for Note {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetNoteScaleValue(new_note) => {
                let prev = self.pitch_name.scale_value;
                self.pitch_name.scale_value = *new_note;
                Action::SetNoteScaleValue(prev)
            }
            Action::SetNoteOctave(octave) => {
                let prev = self.pitch_name.octave;
                self.pitch_name.octave = *octave;
                Action::SetNoteOctave(prev)
            }
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
