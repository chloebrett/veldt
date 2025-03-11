use shared::model::Note;
use shared::pmodel::NoteProto;
use shared::serialize::map_vec;
use shared::save_notes::{
    LoadNotesListRequest, LoadNotesRequest, SaveNotesRequest, load_notes_client::LoadNotesClient,
    load_notes_list_client::LoadNotesListClient, save_notes_client::SaveNotesClient,
};
use tonic_web_wasm_client::Client;
use web_sys::console;

pub async fn save_notes(name: String, notes: Vec<Note>) -> Option<()> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = SaveNotesClient::new(wasm_client);

    let result = grpc
        .save_notes(SaveNotesRequest {
            name,
            notes: notes.iter().map(|&note| note.into()).collect(),
        })
        .await;
    match result {
        Ok(_) => Some(()),
        Err(_) => {
            console::error_1(&"Error saving notes to server.".into());
            None
        }
    }
}

pub async fn load_note_list() -> Option<Vec<String>> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = LoadNotesListClient::new(wasm_client);

    let result = grpc.load_notes_list(LoadNotesListRequest {}).await;
    match result {
        Ok(response) => Some(response.into_inner().names),
        Err(_) => {
            console::error_1(&"Error load note list from server.".into());
            None
        }
    }
}

pub async fn load_notes(name: String) -> Option<Vec<Note>> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = LoadNotesClient::new(wasm_client);

    let result = grpc
        .load_notes(LoadNotesRequest { name })
        .await;

    match result {
        Ok(response) => Some(map_vec::<NoteProto, Note>(response.into_inner().notes)),
        Err(_) => {
            console::error_1(&"Error loading notes from server.".into());
            None
        } 
    }
}
