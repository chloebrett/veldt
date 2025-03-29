use crate::Action;
use shared::logger::log;
use shared::model::Track;

pub fn track_reducer(track: &mut Track, action: &Action) -> Action {
    log(&format!("note_reducer processing: {:?}", action.clone()));

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
