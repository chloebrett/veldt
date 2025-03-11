use shared::consts::XERIC_URL;
use shared::model::Note;
use shared::save_notes::{
    LoadNotesListRequest, LoadNotesRequest, SaveNotesRequest, load_notes_client::LoadNotesClient,
    load_notes_list_client::LoadNotesListClient, save_notes_client::SaveNotesClient,
};
use shared::serialize::map_vec;
use tonic_web_wasm_client::Client;

pub async fn save_notes(name: String, notes: Vec<Note>) {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveNotesClient::new(client);

    let _result = grpc
        .save_notes(SaveNotesRequest {
            name,
            notes: notes.iter().map(|&note| note.into()).collect(),
        })
        .await;
}

pub async fn load_note_list() -> Vec<String> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadNotesListClient::new(client);

    let result = grpc.load_notes_list(LoadNotesListRequest {}).await;
    result.unwrap().into_inner().names
}

pub async fn load_notes(name: String) -> Vec<Note> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadNotesClient::new(client);

    let result = grpc
        .load_notes(LoadNotesRequest { name })
        .await
        .unwrap()
        .into_inner()
        .notes;
    map_vec(result)
}
