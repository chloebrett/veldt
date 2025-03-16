use crate::load_sample::MyLoadSample;
use crate::save::MySaveNotes;
use http::{HeaderValue, Method};
use mesic::render;
use save::ServerSaveTracks;
use shared::bytes::as_bytes;
use shared::consts::{HYDRIC_URL, XERIC_SOCKET_ADDR};
use shared::load_sample::load_sample_server::LoadSampleServer;
use shared::model::MixerChannel;
use shared::render::render_server::{Render, RenderServer};
use shared::render::{RenderReply, RenderRequest};
use shared::save_notes::load_notes_server::LoadNotesServer;
use shared::save_track::load_track_list_server::LoadTrackListServer;
use shared::save_track::load_track_server::LoadTrackServer;
use shared::save_track::save_track_server::SaveTrackServer;
use shared::serialize::map_vec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;

pub mod load_sample;
pub mod save;

struct MyRender;

#[async_trait]
impl Render for MyRender {
    async fn render(
        &self,
        request: tonic::Request<RenderRequest>,
    ) -> Result<tonic::Response<RenderReply>, tonic::Status> {
        let render_data = request.into_inner();
        let track = render_data
            .track
            .ok_or(tonic::Status::invalid_argument("Track must be supplied"))?;
        let effects = render_data.effects;
        let generator = render_data
            .generator
            .ok_or(tonic::Status::invalid_argument("Must supply generator"))?;
        let bpm = render_data.bpm;
        let mixer_channel = MixerChannel {
            effects: map_vec(effects),
        };
        let bytes = as_bytes(&render(
            &track.into(),
            &mixer_channel,
            &generator.into(),
            bpm,
        ));

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let render = RenderServer::new(MyRender);
    let saved_notes = Arc::new(Mutex::new(HashMap::new()));
    let saved_tracks = Arc::new(Mutex::new(HashMap::new()));
    let load_notes = LoadNotesServer::new(MySaveNotes {
        values: Arc::clone(&saved_notes),
    });
    // TODO: stop using the My... pattern? Avoid / call it something else.
    let load_sample = LoadSampleServer::new(MyLoadSample);
    let save_track = SaveTrackServer::new(ServerSaveTracks {
        values: Arc::clone(&saved_tracks),
    });
    let load_track_list = LoadTrackListServer::new(ServerSaveTracks {
        values: Arc::clone(&saved_tracks),
    });
    let load_track = LoadTrackServer::new(ServerSaveTracks {
        values: Arc::clone(&saved_tracks),
    });

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(HYDRIC_URL.parse::<HeaderValue>().unwrap())
                .allow_headers(AllowHeaders::mirror_request())
                .allow_credentials(true),
        )
        .layer(GrpcWebLayer::new())
        .add_service(render)
        .add_service(load_notes)
        .add_service(load_sample)
        .add_service(save_track)
        .add_service(load_track_list)
        .add_service(load_track)
        .serve(*XERIC_SOCKET_ADDR)
        .await?;

    Ok(())
}
