use shared::pmodel::NoteProto;
use shared::save_notes::save_notes_server::SaveNotes;
use shared::save_notes::load_notes_list_server::LoadNotesList;
use shared::save_notes::{SaveNotesReply, SaveNotesRequest, LoadNotesListRequest, LoadNotesListReply};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::async_trait;

pub struct MySaveNotes {
    pub values: SavedNotes,
}
type SavedNotes = Arc<Mutex<HashMap<String, Vec<NoteProto>>>>;

#[async_trait]
impl SaveNotes for MySaveNotes {
    async fn save_notes(
        self: &Self,
        request: tonic::Request<SaveNotesRequest>,
    ) -> Result<tonic::Response<SaveNotesReply>, tonic::Status> {
        let SaveNotesRequest { name, notes } = request.into_inner();
        self.values
            .lock()
            .unwrap()
            .insert(name.clone(), notes.clone());
        println!("Saved {}", name.clone());
        Ok(tonic::Response::new(SaveNotesReply {}))
    }
}

#[async_trait]
impl LoadNotesList for MySaveNotes {
    async fn load_notes_list(
        self: & Self,
        _request: tonic::Request<LoadNotesListRequest>,
    ) -> Result<tonic::Response<LoadNotesListReply>, tonic::Status> {
        let list = self.values.lock().unwrap().keys().cloned().collect();
        Ok(tonic::Response::new(LoadNotesListReply {names: list}))
    }
}
