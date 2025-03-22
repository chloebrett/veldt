use crate::state::Action;
use ordered_float::OrderedFloat;
use shared::model::PlacedNote;
use web_sys::console;

pub fn note_reducer(note: &mut PlacedNote, action: &Action) -> Action {
    console::log_1(&format!("note_reducer processing: {:?}", action.clone()).into());

    match action {
        Action::SetNoteScaleValue(new_note) => {
            let prev = note.note.pitch_name.scale_value;
            note.note.pitch_name.scale_value = *new_note;
            Action::SetNoteScaleValue(prev)
        }
        Action::SetNoteOctave(octave) => {
            let prev = note.note.pitch_name.octave;
            note.note.pitch_name.octave = *octave;
            Action::SetNoteOctave(prev)
        }
        Action::SetNoteDuration(duration) => {
            let prev = note.note.beats;
            note.note.beats = *duration;
            Action::SetNoteDuration(prev)
        }
        Action::SetNoteOffset(offset) => {
            let prev = note.offset;
            note.offset = OrderedFloat(*offset);
            Action::SetNoteOffset(*prev)
        }
        _ => Action::NonReversible,
    }
}
