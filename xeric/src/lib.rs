use crate::save::MySaveNotes;
use http::{HeaderValue, Method};
use mesic::render;
use shared::bytes::as_bytes;
use shared::consts::{HYDRIC_URL, XERIC_SOCKET_ADDR};
use shared::render::render_server::{Render, RenderServer};
use shared::render::{RenderReply, RenderRequest};
use shared::save_notes::load_notes_list_server::LoadNotesListServer;
use shared::save_notes::load_notes_server::LoadNotesServer;
use shared::save_notes::save_notes_server::SaveNotesServer;
use shared::serialize::map_vec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;

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
        let bytes = as_bytes(&render(
            &track.into(),
            map_vec(effects),
            generator.into(),
            bpm,
        ));

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let render = RenderServer::new(MyRender);
    let saved_notes = Arc::new(Mutex::new(HashMap::new()));
    let save_notes = SaveNotesServer::new(MySaveNotes {
        values: Arc::clone(&saved_notes),
    });
    let load_notes_list = LoadNotesListServer::new(MySaveNotes {
        values: Arc::clone(&saved_notes),
    });
    let load_notes = LoadNotesServer::new(MySaveNotes {
        values: Arc::clone(&saved_notes),
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
        .add_service(save_notes)
        .add_service(load_notes_list)
        .add_service(load_notes)
        .serve(*XERIC_SOCKET_ADDR)
        .await?;

    Ok(())
}
