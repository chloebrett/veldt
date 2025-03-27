use shared::consts::XERIC_URL;
use shared::load_sample::{LoadSampleRequest, load_sample_client::LoadSampleClient};
use shared::model::Sample;
use tonic_web_wasm_client::Client;
use shared::logger::error;

pub async fn load_sample(filename: String) -> Option<Sample> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadSampleClient::new(client);

    let result = grpc.load_sample(LoadSampleRequest { filename }).await;

    if let Ok(load_sample_reply) = result {
        if let Some(sample) = load_sample_reply.into_inner().sample {
            return Some(sample.into());
        }
    }
    error("Error loading sample from server.".to_string());
    None
}
