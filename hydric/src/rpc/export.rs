use shared::consts::XERIC_URL;
use shared::export::ExportRequest;
use shared::export::export_client::ExportClient;
use shared::model::Project;
use tonic_web_wasm_client::Client;

pub async fn export(project: Project) -> Result<(), tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = ExportClient::new(client);

    let result = grpc
        .export(ExportRequest {
            project: Some(project.into()),
        })
        .await;

    result.map(|_| ())
}

pub async fn export_mp3(project: Project) -> Result<(), tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = ExportClient::new(client);

    let result = grpc
        .export_mp3(ExportRequest {
            project: Some(project.into()),
        })
        .await;

    result.map(|_| ())
}
