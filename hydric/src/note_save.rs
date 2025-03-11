use shared::model::Note;
use shared::save_notes::{
    LoadNotesListRequest, LoadNotesRequest, SaveNotesRequest, load_notes_client::LoadNotesClient,
    load_notes_list_client::LoadNotesListClient, save_notes_client::SaveNotesClient,
};
use tonic_web_wasm_client::Client;

pub async fn save_notes(name: String, notes: Vec<Note>) -> Result<(),String> {
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
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Error saving to server."))
    }
}

pub async fn load_note_list() -> Vec<String> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = LoadNotesListClient::new(wasm_client);

    let result = grpc.load_notes_list(LoadNotesListRequest {}).await;
    result.unwrap().into_inner().names
}

pub async fn load_notes(name: String) -> Vec<Note> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = LoadNotesClient::new(wasm_client);

    let result = grpc
        .load_notes(LoadNotesRequest { name })
        .await
        .unwrap()
        .into_inner()
        .notes;
    result.iter().map(|&note| Note::from(note)).collect()
}
