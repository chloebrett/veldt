use shared::consts::XERIC_URL;
use shared::model::Track;
use shared::save_track::load_track_client::LoadTrackClient;
use shared::save_track::load_track_list_client::LoadTrackListClient;
use shared::save_track::save_track_client::SaveTrackClient;
use shared::save_track::{LoadTrackListRequest, LoadTrackRequest, SaveTrackRequest};
use tonic_web_wasm_client::Client;
use shared::logger::error;

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
            error("Error saving notes to server.");
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
            error("Error loading track list from server.");
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
    error("Error loading track list from server.");
    None
}
