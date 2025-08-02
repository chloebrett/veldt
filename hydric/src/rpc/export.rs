use shared::consts::XERIC_URL;
use shared::export::ExportRequest;
use shared::export::export_client::ExportClient;
use shared::model::Project;
use shared::pmodel::ExportConfigProto;
use tonic_web_wasm_client::Client;

pub async fn export(project: Project, audio_file_type: String) -> Result<(), tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = ExportClient::new(client);

    let result = grpc
        .export(ExportRequest {
            project: Some(project.into()),
            config: Some(ExportConfigProto {
                audio_type: audio_file_type,
            }),
        })
        .await;

    result.map(|_| ())
}
