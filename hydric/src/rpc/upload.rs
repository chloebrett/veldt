use shared::consts::XERIC_URL;
use shared::upload::UploadSampleRequest;
use tonic_web_wasm_client::Client;

pub async fn upload_sample(file_name: String,bytes: web::bytes) -> Result<(), ()>{
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveLoadClient::new(client);

    let result = grpc.upload_sample(UploadSampleRequest {
        name: file_name,
        data: bytes,
    })
    .await;

    result.map(|_| ()).map_err(|_| ())
}