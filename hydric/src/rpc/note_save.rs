use shared::consts::XERIC_URL;
use shared::model::Track;
use shared::save_track::load_track_client::LoadTrackClient;
use shared::save_track::load_track_list_client::LoadTrackListClient;
use shared::save_track::save_track_client::SaveTrackClient;
use shared::save_track::{LoadTrackListRequest, LoadTrackRequest, SaveTrackRequest};
use tonic_web_wasm_client::Client;
use web_sys::console;

pub async fn save_track(name: String, track: Track) -> Option<()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveTrackClient::new(client);

    let result = grpc
        .save_track(SaveTrackRequest {
            name,
            track: Some(track.into()),
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

pub async fn load_track_list() -> Option<Vec<String>> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadTrackListClient::new(client);

    let result = grpc.load_track_list(LoadTrackListRequest {}).await;
    match result {
        Ok(response) => Some(response.into_inner().names),
        Err(_) => {
            console::error_1(&"Error loading track list from server.".into());
            None
        }
    }
}

pub async fn load_track(name: String) -> Option<Track> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadTrackClient::new(client);

    let result = grpc.load_track(LoadTrackRequest { name }).await;

    if let Ok(load_track_reply) = result {
        if let Some(track) = load_track_reply.into_inner().track {
            return Some(track.into());
        }
    }
    console::error_1(&"Error loading track from server.".into());
    None
}
