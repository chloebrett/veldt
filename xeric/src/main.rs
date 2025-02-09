use std::net::SocketAddr;

use shared::echo_server::{Echo, EchoServer};
use shared::{EchoReply, EchoRequest};
use tonic::async_trait;

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

    tonic::transport::Server::builder()
        .add_service(EchoServer::new(MyEcho))
        .serve(addr)
        .await?;

    Ok(())
}
