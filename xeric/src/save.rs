use shared::save_notes::save_notes_server::SaveNotes;
use shared::pmodel::NoteProto;
use shared::save_notes::{SaveNotesReply, SaveNotesRequest};
use std::collections::HashMap;
use tonic::async_trait;
use std::sync::{Arc, Mutex};

pub struct MySaveNotes {
    pub values: SavedNotes
}
type SavedNotes = Arc<Mutex<HashMap::<String, Vec<NoteProto>>>>;

#[async_trait]
impl SaveNotes for MySaveNotes {
    async fn save_notes(
        self: & Self,
        request: tonic::Request<SaveNotesRequest>,
    ) -> Result<tonic::Response<SaveNotesReply>, tonic::Status> {
        let SaveNotesRequest {name, notes} = request
            .into_inner();
        self.values.lock().unwrap().insert(
            name.clone(),
            notes.clone()
            ); 
        println!("Saved {}", name.clone());
        Ok(tonic::Response::new(SaveNotesReply {reply: String::from("Ok")}))
    }
}
