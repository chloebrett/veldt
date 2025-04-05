use shared::consts::XERIC_URL;
use shared::model::Project;
use shared::save_load::load_project_client::LoadProjectClient;
use shared::save_load::load_project_list_client::LoadProjectListClient;
use shared::save_load::save_project_client::SaveProjectClient;
use shared::save_load::{LoadProjectListRequest, LoadProjectRequest, SaveProjectRequest};
use tonic_web_wasm_client::Client;

pub async fn save_project(name: String, project: Project) -> Result<(), ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveProjectClient::new(client);

    let result = grpc
        .save_project(SaveProjectRequest {
            name,
            project: Some(project.into()),
        })
        .await;

    result.map(|_| ()).map_err(|_| ())
}

pub async fn load_project_list() -> Result<Vec<String>, ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadProjectListClient::new(client);

    let result = grpc.load_project_list(LoadProjectListRequest {}).await;

    result
        .map(|it| it.into_inner().project_names)
        .map_err(|_| ())
}

pub async fn load_project(name: String) -> Result<Project, ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadProjectClient::new(client);

    let result = grpc.load_project(LoadProjectRequest { name }).await;

    result
        .map(|it| it.into_inner().project.unwrap().into())
        .map_err(|_| ())
}
