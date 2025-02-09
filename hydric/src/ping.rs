use clap::Parser;
use shared::{echo_client::EchoClient, EchoRequest};
use tonic_web_wasm_client::Client;

#[derive(Parser, Debug)]
struct Opt {
    /// Server to connect to
    #[clap(long, default_value = "http://localhost:3000")]
    server: String,
    /// Message to send
    message: String,
}

#[tokio::main(flavor = "current_thread")]
pub async fn ping() {
    let base_url = "http://localhost:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = EchoClient::new(wasm_client);

    let res = grpc
        .echo(EchoRequest {
            message: "PING".to_string(),
        })
        .await
        .unwrap();

    println!("{:?}", res);
}
