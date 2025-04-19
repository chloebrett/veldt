use shared::consts::XERIC_URL;
use shared::load_sample::{
    LoadSampleRequest, LoadSampleTreeRequest, load_sample_client::LoadSampleClient,
};
use shared::model::{FilenameTree, Sample};
use tonic_web_wasm_client::Client;

pub async fn load_sample(filename: String) -> Result<Sample, ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadSampleClient::new(client);

    let result = grpc.load_sample(LoadSampleRequest { filename }).await;

    result
        .map(|it| it.into_inner().sample.unwrap().into())
        .map_err(|_| ())
}

pub async fn load_sample_tree() -> Result<FilenameTree, ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadSampleClient::new(client);

    let result = grpc.load_sample_tree(LoadSampleTreeRequest {}).await;

    result
        .map(|it| it.into_inner().tree.unwrap().into())
        .map_err(|_| ())
}
