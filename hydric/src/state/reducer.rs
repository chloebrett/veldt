use super::{Action, Store};
use ordered_float::OrderedFloat;

pub fn reducer(store: &mut Store, action: Action) {
    match action {
        Action::SetKey(key) => store.key = key,
        Action::SetScale(scale) => store.scale = scale,
        Action::SetBpm(bpm) => store.project.bpm = bpm,
        Action::SetNoteScaleValue {
            track_index,
            note_index,
            note,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.pitch_name.scale_value = note
        }
        Action::SetNoteOctave {
            track_index,
            note_index,
            octave,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.pitch_name.octave = octave
        }
        Action::SetNoteDuration {
            track_index,
            note_index,
            duration,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.beats = duration
        }
        Action::SetNoteOffset {
            track_index,
            note_index,
            offset,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].offset = OrderedFloat(offset)
        }
        Action::DeleteNote {
            track_index,
            note_index,
        } => {
            store.project.tracks[track_index].notes.remove(note_index);
        }
        Action::AddNote { track_index, note } => {
            store.project.tracks[track_index].notes.push(note);
        }
    }
}
