use crate::save::MySaveNotes;
use http::{HeaderValue, Method};
use mesic::{SupersawConfig, render};
use shared::bytes::as_bytes;
use shared::model::{
    AdsrEnvelope, GeneratorInstance, GeneratorMeta, GeneratorType, SimpleWaveConfig, WaveType,
};
use shared::render::render_server::{Render, RenderServer};
use shared::render::{RenderReply, RenderRequest};
use shared::save_notes::load_notes_list_server::LoadNotesListServer;
use shared::save_notes::save_notes_server::SaveNotesServer;
use std::collections::HashMap;
use std::net::SocketAddr;
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
        let supersaw_config = SupersawConfig {
            osc_count: 1,

            detune_cents: 0.0,
        };
        let generator = GeneratorInstance {
            id: 0,

            kind: GeneratorType::SimpleWave {
                config: SimpleWaveConfig {
                    wave: WaveType::Saw,

                    envelope: AdsrEnvelope {
                        attack: 0.1,
                        decay: 0.1,
                        sustain: 0.8,
                        release: 0.1,
                    },
                },
            },

            meta: GeneratorMeta { volume: 1.0 },
        };
        let bpm = 120.0;

        let track = request
            .into_inner()
            .track
            .ok_or(tonic::Status::invalid_argument("Track must be supplied"))?;
        let bytes = as_bytes(&render(
            &track.into(),
            supersaw_config,
            vec![],
            generator,
            bpm,
        ));

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let render = RenderServer::new(MyRender);
    let saved_notes = Arc::new(Mutex::new(HashMap::new()));
    let save_notes = SaveNotesServer::new(MySaveNotes {
        values: Arc::clone(&saved_notes),
    });
    let load_notes_list = LoadNotesListServer::new(MySaveNotes {
        values: Arc::clone(&saved_notes),
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
        .add_service(load_notes_list)
        .serve(addr)
        .await?;

    Ok(())
}
