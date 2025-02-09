use std::net::SocketAddr;

use shared::echo_server::{Echo, EchoServer};
use shared::{EchoReply, EchoRequest};
use tonic::async_trait;
use tonic_web::GrpcWebLayer;
use tower_http::cors::AllowHeaders;
use http::{Method, HeaderValue};

struct MyEcho;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000)).into();

    let echo = EchoServer::new(MyEcho);

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
        .serve(addr)
        .await?;

    Ok(())
}
