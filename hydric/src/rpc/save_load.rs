use shared::consts::XERIC_URL;
use shared::model::Project;
use shared::save_load::save_load_client::SaveLoadClient;
use shared::save_load::{LoadProjectListRequest, LoadProjectRequest, SaveProjectRequest};
use tonic_web_wasm_client::Client;

pub async fn save_project(project: Project) -> Result<(), tonic::Status> {
    // TODO: recycle clients?
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveLoadClient::new(client);

    let result = grpc
        .save_project(SaveProjectRequest {
            name: project.name.clone(),
            project: Some(project.into()),
        })
        .await;

    result.map(|_| ())
}

pub async fn load_project_list() -> Result<Vec<String>, tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveLoadClient::new(client);

    let result = grpc.load_project_list(LoadProjectListRequest {}).await;

    result.map(|it| it.into_inner().project_names)
}

pub async fn load_project(name: String) -> Result<Project, tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveLoadClient::new(client);

    let result = grpc.load_project(LoadProjectRequest { name }).await;

    result.map(|it| it.into_inner().project.unwrap().into())
}
