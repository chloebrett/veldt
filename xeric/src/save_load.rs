use shared::logger::log;
use shared::model::Project;
use shared::save_load::load_project_list_server::LoadProjectList;
use shared::save_load::load_project_server::LoadProject;
use shared::save_load::save_project_server::SaveProject;
use shared::save_load::{
    LoadProjectListReply, LoadProjectListRequest, LoadProjectReply, LoadProjectRequest,
    SaveProjectReply, SaveProjectRequest,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::async_trait;

#[derive(Clone)]
pub struct SaveLoadContext {
    pub projects: SavedProjects,
}

type SavedProjects = Arc<Mutex<HashMap<String, Project>>>;

#[async_trait]
impl SaveProject for SaveLoadContext {
    async fn save_project(
        self: &Self,
        request: tonic::Request<SaveProjectRequest>,
    ) -> Result<tonic::Response<SaveProjectReply>, tonic::Status> {
        let SaveProjectRequest { name, project } = request.into_inner();
        self.projects
            .lock()
            .unwrap()
            .insert(name.clone(), project.unwrap().clone().into());
        log(&format!("Saved {}", name.clone()));
        Ok(tonic::Response::new(SaveProjectReply {}))
    }
}

#[async_trait]
impl LoadProjectList for SaveLoadContext {
    async fn load_project_list(
        self: &Self,
        _request: tonic::Request<LoadProjectListRequest>,
    ) -> Result<tonic::Response<LoadProjectListReply>, tonic::Status> {
        let list = self.projects.lock().unwrap().keys().cloned().collect();
        Ok(tonic::Response::new(LoadProjectListReply {
            project_names: list,
        }))
    }
}

#[async_trait]
impl LoadProject for SaveLoadContext {
    async fn load_project(
        self: &Self,
        request: tonic::Request<LoadProjectRequest>,
    ) -> Result<tonic::Response<LoadProjectReply>, tonic::Status> {
        let name = request.into_inner().name;
        if let Some(project) = self.projects.lock().unwrap().get(&name) {
            Ok(tonic::Response::new(LoadProjectReply {
                project: Some(project.clone().into()),
            }))
        } else {
            Err(tonic::Status::invalid_argument(
                "Project name was not found on server.",
            ))
        }
    }
}
