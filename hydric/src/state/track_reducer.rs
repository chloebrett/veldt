use crate::state::Action;
use shared::model::Track;
use web_sys::console;

pub fn track_reducer(track: &mut Track, action: &Action) -> Action {
    console::log_1(&format!("note_reducer processing: {:?}", action.clone()).into());

    match action {
        Action::DeleteNote { note_index } => {
            let prev = track
                .notes
                .get(*note_index)
                .expect("Can't delete non-existent track!")
                .clone();
            track.notes.remove(*note_index);
            Action::AddNote(prev)
        }
        Action::AddNote(note) => {
            let index = track.notes.len();
            track.notes.push(note.clone());
            Action::DeleteNote { note_index: index }
        }
        _ => Action::NonReversible,
    }
}
