use shared::consts::XERIC_URL;
use shared::upload::UploadSampleRequest;
use shared::upload::upload_client::UploadClient;
use tonic_web_wasm_client::Client;

pub async fn upload_sample(file_name: String, bytes: Vec<u8>) -> Result<(), ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = UploadClient::new(client);

    let result = grpc
        .upload_sample(UploadSampleRequest {
            name: file_name,
            uploaded_sample: bytes,
        })
        .await;

    result.map(|_| ()).map_err(|_| ())
}
