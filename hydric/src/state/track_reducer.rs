use super::{note_index, note_reducer};
use crate::state::Action;
use shared::model::Track;
use std::cell::RefMut;
use web_sys::console;

pub fn track_reducer(track: &mut Track, action: &Action) {
    console::log_1(&format!("note_reducer processing: {:?}", action.clone()).into());

    if let Some(note_index) = note_index(action) {
        return note_reducer(&mut track.notes[note_index], action);
    }

    match action {
        Action::DeleteNote { note_index, .. } => {
            track.notes.remove(*note_index);
        }
        Action::AddNote { note, .. } => {
            track.notes.push(note.clone());
        }
        _ => {}
    };
}
