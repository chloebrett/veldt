use super::{Action, Store};

pub fn reducer(store: &mut Store, action: Action) {
    match action {
        Action::SetKey(key) => store.key = key,
        Action::SetScale(scale) => store.scale = scale,
        Action::SetBpm(bpm) => store.project.bpm = bpm,
        Action::SetNote {
            track_index,
            note_index,
            note,
        } => {
            let notes = &mut store.project.tracks[track_index].notes;
            notes[note_index].note.pitch_name.scale_value = note
        }
    }
}
