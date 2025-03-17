use crate::state::Action;
use shared::model::Track;
use web_sys::console;

pub fn track_reducer(track: &mut Track, action: &Action) {
    console::log_1(&format!("note_reducer processing: {:?}", action.clone()).into());

    match action {
        Action::DeleteNote { note_index } => {
            track.notes.remove(*note_index);
        }
        Action::AddNote(note) => {
            track.notes.push(note.clone());
        }
        _ => {}
    };
}
