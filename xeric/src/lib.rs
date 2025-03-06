use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Mutex;

use http::{HeaderValue, Method};
use mesic::{SupersawConfig, render};
use shared::bytes::as_bytes;
use shared::render::render_server::{Render, RenderServer};
use shared::save_notes::save_notes_server::{SaveNotes, SaveNotesServer};
use shared::pmodel::NoteProto;
use shared::save_notes::{SaveNotesReply, SaveNotesRequest};
use shared::render::{RenderReply, RenderRequest};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;


struct MyRender;
struct MySaveNotes {
    values: SavedNotes
}
type SavedNotes = Mutex<HashMap::<String, Vec<NoteProto>>>;

#[async_trait]
impl SaveNotes for MySaveNotes {
    async fn save_notes(
        self: & Self,
        request: tonic::Request<SaveNotesRequest>,
    ) -> Result<tonic::Response<SaveNotesReply>, tonic::Status> {
        let SaveNotesRequest {name, notes} = request
            .into_inner();
        self.values.lock().unwrap().insert(
            name,
            notes
            ); 
        Ok(tonic::Response::new(SaveNotesReply {reply: String::from("Ok")}))
    }
}

#[async_trait]
impl Render for MyRender {
    async fn render(
        &self,
        request: tonic::Request<RenderRequest>,
    ) -> Result<tonic::Response<RenderReply>, tonic::Status> {
        let supersaw_config = SupersawConfig {
            osc_count: 1,

            detune_cents: 0.0,
        };

        let track = request
            .into_inner()
            .track
            .ok_or(tonic::Status::invalid_argument("Track must be supplied"))?;
        let bytes = as_bytes(&render(&track.into(), supersaw_config));

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let render = RenderServer::new(MyRender);
    let save_notes = SaveNotesServer::new(MySaveNotes {
        values: Mutex::new(HashMap::new())
    });

    tonic::transport::Server::builder()
        .accept_http1(true)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap())
                .allow_headers(AllowHeaders::mirror_request())
                .allow_credentials(true),
        )
        .layer(GrpcWebLayer::new())
        .add_service(render)
        .add_service(save_notes)
        .serve(addr)
        .await?;

    Ok(())
}
