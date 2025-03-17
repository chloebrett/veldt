use crate::state::Action;
use ordered_float::OrderedFloat;
use shared::model::PlacedNote;
use web_sys::console;

pub fn note_index(action: &Action) -> Option<usize> {
    match action {
        Action::SetNoteScaleValue { note_index, .. } => Some(*note_index),
        Action::SetNoteOctave { note_index, .. } => Some(*note_index),
        Action::SetNoteOffset { note_index, .. } => Some(*note_index),
        Action::SetNoteDuration { note_index, .. } => Some(*note_index),
        Action::DeleteNote { note_index, .. } => Some(*note_index),
        _ => None,
    }
}

pub fn note_reducer(note: &mut PlacedNote, action: &Action) {
    console::log_1(&format!("note_reducer processing: {:?}", action.clone()).into());

    match action {
        Action::SetNoteScaleValue { note: new_note, .. } => {
            note.note.pitch_name.scale_value = *new_note
        }
        Action::SetNoteOctave { octave, .. } => note.note.pitch_name.octave = *octave,
        Action::SetNoteDuration { duration, .. } => note.note.beats = *duration,
        Action::SetNoteOffset { offset, .. } => note.offset = OrderedFloat(*offset),
        _ => {}
    }
}
