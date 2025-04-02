use shared::consts::XERIC_URL;
use shared::logger::error;
use shared::model::Project;
use shared::save_load::load_project_client::LoadProjectClient;
use shared::save_load::load_project_list_client::LoadProjectListClient;
use shared::save_load::save_project_client::SaveProjectClient;
use shared::save_load::{LoadProjectListRequest, LoadProjectRequest, SaveProjectRequest};
use tonic_web_wasm_client::Client;

pub async fn save_project(name: String, project: Project) -> Option<()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = SaveProjectClient::new(client);

    let result = grpc
        .save_project(SaveProjectRequest {
            name,
            project: Some(project.into()),
        })
        .await;
    match result {
        Ok(_) => Some(()),
        Err(_) => {
            error("Error saving notes to server.");
            None
        }
    }
}

pub async fn load_project_list() -> Option<Vec<String>> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadProjectListClient::new(client);

    let result = grpc.load_project_list(LoadProjectListRequest {}).await;
    match result {
        Ok(response) => Some(response.into_inner().project_names),
        Err(_) => {
            error("Error loading project list from server.");
            None
        }
    }
}

pub async fn load_project(name: String) -> Option<Project> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = LoadProjectClient::new(client);

    let result = grpc.load_project(LoadProjectRequest { name }).await;

    if let Ok(load_project_reply) = result {
        if let Some(project) = load_project_reply.into_inner().project {
            return Some(project.into());
        }
    }
    error("Error loading project list from server.");
    None
}
