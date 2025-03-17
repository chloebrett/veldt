use super::{Action, Store};
use shared::model::{Note, PlacedNote};

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
            // TODO: don't do this, index by key in the set instead? Have a map too? Not sure yet.
            let mut note: &mut PlacedNote = notes.iter().nth(note_index);

            notes_vec[note_index].pitch_name.scale_value = note
        }
    }
}
