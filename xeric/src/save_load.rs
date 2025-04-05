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

// Save/load RPCs share some stateful context. Currently, we don't save/load to a file, we just
// store the saved projects in memory while the server is running.
#[derive(Clone)]
pub struct SaveLoadContext {
    // Since SavedProjects is an Arc<Mutex<...>>, cloning the SaveLoadContext just clones the
    // reference; the data is still shared.
    pub projects: SavedProjects,
}

// Using an Arc<Mutex<...>> because the hashmap may be accessed from multiple threads.
type SavedProjects = Arc<Mutex<HashMap<String, Project>>>;

#[async_trait]
impl SaveProject for SaveLoadContext {
    /// Saves a project to server memory by name.
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
    /// Loads the list of project names that are saved. LoadProject can then be called to load an
    /// actual project.
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
    /// Loads a project by name. If it doesn't exist in the server memory, returns an error.
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
