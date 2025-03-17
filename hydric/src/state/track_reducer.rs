pub fn track_reducer(mut track: RefMut<'_, Track>, action: Action) {
    if let Some(note_index) = note_index(action) {
        return note_reducer(track.notes[note_index], action)
    }

    match action {
        Action::SetNoteScaleValue {
            note_index, note, ..
        } => track.notes[note_index].note.pitch_name.scale_value = note,
        Action::SetNoteOctave {
            note_index, octave, ..
        } => track.notes[note_index].note.pitch_name.octave = octave,
        Action::SetNoteDuration {
            note_index,
            duration,
            ..
        } => track.notes[note_index].note.beats = duration,
        Action::SetNoteOffset {
            note_index, offset, ..
        } => track.notes[note_index].offset = OrderedFloat(offset),
        Action::DeleteNote { note_index, .. } => track.notes.remove(note_index),
        Action::AddNote { note, .. } => track.notes.push(note),
        _ => {}
    }
}
