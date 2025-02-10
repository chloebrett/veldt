use std::net::SocketAddr;

use http::{HeaderValue, Method};
use mesic::io::as_bytes;
use mesic::render;
use shared::echo::echo_server::{Echo, EchoServer};
use shared::echo::{EchoReply, EchoRequest};
use shared::render::render_server::{Render, RenderServer};
use shared::render::{RenderReply, RenderRequest};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;

struct MyRender;

struct MyEcho;

#[async_trait]
impl Render for MyRender {
    async fn render(
        &self,
        request: tonic::Request<RenderRequest>,
    ) -> Result<tonic::Response<RenderReply>, tonic::Status> {
        let track = request
            .into_inner()
            .track
            .ok_or(tonic::Status::invalid_argument("Track must be supplied"))?;
        let bytes = as_bytes(&render(&track.into()));

        Ok(tonic::Response::new(RenderReply { audio: bytes }))
    }
}

#[async_trait]
impl Echo for MyEcho {
    async fn echo(
        &self,
        request: tonic::Request<EchoRequest>,
    ) -> Result<tonic::Response<EchoReply>, tonic::Status> {
        Ok(tonic::Response::new(EchoReply {
            message: format!("Echoing back: {}", request.get_ref().message),
        }))
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let echo = EchoServer::new(MyEcho);
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
        .add_service(echo)
        .add_service(render)
        .serve(addr)
        .await?;

    Ok(())
}
