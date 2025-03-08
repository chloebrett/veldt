use std::net::SocketAddr;

use http::{HeaderValue, Method};
use mesic::{SupersawConfig, render};
use shared::bytes::as_bytes;
use shared::render::render_server::{Render, RenderServer};
use shared::render::{RenderReply, RenderRequest};
use shared::types::{Freq, KnobPosition};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;

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
        let resonant_freq: Freq = 1000.0;
        let resonance_q: KnobPosition = 1.0;
        let resonance_wet: KnobPosition = 1.0;

        let track = request
            .into_inner()
            .track
            .ok_or(tonic::Status::invalid_argument("Track must be supplied"))?;
        let bytes = as_bytes(&render(
            &track.into(),
            supersaw_config,
            resonant_freq,
            resonance_q,
            resonance_wet,
        ));

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let render = RenderServer::new(MyRender);

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
        .serve(addr)
        .await?;

    Ok(())
}
