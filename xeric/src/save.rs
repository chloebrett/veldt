use shared::pmodel::{NoteProto, TrackProto};
use shared::save_notes::load_notes_server::LoadNotes;
use shared::save_notes::{
    LoadNotesReply, LoadNotesRequest
};
use shared::save_track::save_track_server::SaveTrack;
use shared::save_track::load_track_list_server::LoadTrackList;
use shared::save_track::{LoadTrackListRequest, LoadTrackListReply, SaveTrackReply, SaveTrackRequest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::async_trait;

pub struct MySaveNotes {
    pub values: SavedNotes,
}
type SavedNotes = Arc<Mutex<HashMap<String, Vec<NoteProto>>>>;

pub struct ServerSaveTracks {
    pub values: SavedTracks,
}
type SavedTracks = Arc<Mutex<HashMap<String, TrackProto>>>;

#[async_trait]
impl SaveTrack for ServerSaveTracks {
    async fn save_track(
        self: &Self,
        request: tonic::Request<SaveTrackRequest>,
    ) -> Result<tonic::Response<SaveTrackReply>, tonic::Status> {
        let SaveTrackRequest { name, track } = request.into_inner();
        self.values
            .lock()
            .unwrap()
            .insert(name.clone(), track.unwrap().clone());
        println!("Saved {}", name.clone());
        Ok(tonic::Response::new(SaveTrackReply {}))
    }
}

#[async_trait]
impl LoadTrackList for ServerSaveTracks {
    async fn load_track_list (
        self: &Self,
        _request: tonic::Request<LoadTrackListRequest>,
    ) -> Result<tonic::Response<LoadTrackListReply>, tonic::Status> {
        let list = self.values.lock().unwrap().keys().cloned().collect();
        Ok(tonic::Response::new(LoadTrackListReply { names: list }))
    }
}

#[async_trait]
impl LoadNotes for MySaveNotes {
    async fn load_notes(
        self: &Self,
        request: tonic::Request<LoadNotesRequest>,
    ) -> Result<tonic::Response<LoadNotesReply>, tonic::Status> {
        let name = request.into_inner().name;
        if let Some(notes) = self.values.lock().unwrap().get(&name) {
            Ok(tonic::Response::new(LoadNotesReply {
                notes: notes.to_vec(),
            }))
        } else {
            // TODO Handle error better.
            Ok(tonic::Response::new(LoadNotesReply { notes: vec![] }))
        }
    }
}
