use leptos::logging::log;
use shared::{EchoRequest, echo_client::EchoClient};
use tonic_web_wasm_client::Client;

pub async fn ping() {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = EchoClient::new(wasm_client);

    let result = grpc
        .echo(EchoRequest {
            message: "PING".to_string(),
        })
        .await
        .unwrap();

    log!("{:?}", result);
}
