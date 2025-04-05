use shared::logger::log;
use shared::pmodel::ProjectProto;
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

type SavedProjects = Arc<Mutex<HashMap<String, ProjectProto>>>;

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
            .insert(name.clone(), project.unwrap().clone());
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
                project: Some(project.clone()),
            }))
        } else {
            Err(tonic::Status::invalid_argument(
                "Project name was not found on server.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use shared::model::Project;

    use super::*;

    fn empty_project(name: String) -> Project {
        Project {
            name,
            tracks: vec![],
            track_placements: vec![],
            samples: vec![],
            generators: vec![],
            mixer: vec![],
            bpm: 120.0,
        }
    }

    #[tokio::test]
    async fn project_save_round_trip() {
        // ARRANGE
        let save_load_context = SaveLoadContext {
            projects: Arc::new(Mutex::new(HashMap::new())),
        };
        let project_name = "test";
        let project_proto: ProjectProto = empty_project(project_name.into()).into();
        let save_request = tonic::Request::new(SaveProjectRequest {
            name: project_name.into(),
            project: Some(project_proto.clone()),
        });
        let load_request = tonic::Request::new(LoadProjectRequest {
            name: project_name.into(),
        });

        // ACT
        let _ = save_load_context.save_project(save_request).await;
        let response = save_load_context.load_project(load_request).await;
        let load_project_proto = response.unwrap().into_inner().project.unwrap();

        // ASSERT
        assert_eq!(load_project_proto, project_proto)
    }

    #[tokio::test]
    async fn load_project_list() {
        // ARRANGE
        let save_load_context = SaveLoadContext {
            projects: Arc::new(Mutex::new(HashMap::new())),
        };
        let project_name_1 = "A";
        let project_name_2 = "B";
        // Create set to compare to response agnostic of order.
        let name_set: HashSet<String> =
            HashSet::from_iter(vec![project_name_1.into(), project_name_2.into()]);
        let project_proto_1: ProjectProto = empty_project(project_name_1.into()).into();
        let project_proto_2: ProjectProto = empty_project(project_name_2.into()).into();
        let save_request_1 = tonic::Request::new(SaveProjectRequest {
            name: project_name_1.into(),
            project: Some(project_proto_1.clone()),
        });
        let save_request_2 = tonic::Request::new(SaveProjectRequest {
            name: project_name_2.into(),
            project: Some(project_proto_2.clone()),
        });
        let load_request = tonic::Request::new(LoadProjectListRequest {});

        // ACT
        let _ = save_load_context.save_project(save_request_1).await;
        let _ = save_load_context.save_project(save_request_2).await;
        let response = save_load_context.load_project_list(load_request).await;
        let project_list = response.unwrap().into_inner().project_names;

        // ASSERT
        let loaded_set: HashSet<String> = HashSet::from_iter(project_list);
        assert_eq!(loaded_set, name_set)
    }

    #[tokio::test]
    async fn load_unsaved_track_name_fails() {
        // ARRANGE
        let save_load_context = SaveLoadContext {
            projects: Arc::new(Mutex::new(HashMap::new())),
        };
        let project_name = "test";
        let unsaved_name = "unsaved";
        let project_proto: ProjectProto = empty_project(project_name.into()).into();
        let save_request = tonic::Request::new(SaveProjectRequest {
            name: project_name.into(),
            project: Some(project_proto.clone()),
        });
        let load_request = tonic::Request::new(LoadProjectRequest {
            name: unsaved_name.into(),
        });

        // ACT
        let _ = save_load_context.save_project(save_request).await;
        let response = save_load_context.load_project(load_request).await;

        // ASSERT
        assert!(response.is_err())
    }

    #[tokio::test]
    async fn save_same_project_name_overwrites_project() {
        // ARRANGE
        let save_load_context = SaveLoadContext {
            projects: Arc::new(Mutex::new(HashMap::new())),
        };
        let project_name = "A";
        let project_proto_1: ProjectProto = Project {
            name: project_name.into(),
            tracks: vec![],
            track_placements: vec![],
            samples: vec![],
            generators: vec![],
            mixer: vec![],
            bpm: 120.0,
        }
        .into();
        let project_proto_2: ProjectProto = Project {
            name: project_name.into(),
            tracks: vec![],
            track_placements: vec![],
            samples: vec![],
            generators: vec![],
            mixer: vec![],
            bpm: 60.0,
        }
        .into();
        let save_request_1 = tonic::Request::new(SaveProjectRequest {
            name: project_name.into(),
            project: Some(project_proto_1.clone()),
        });
        let save_request_2 = tonic::Request::new(SaveProjectRequest {
            name: project_name.into(),
            project: Some(project_proto_2.clone()),
        });
        let load_request = tonic::Request::new(LoadProjectRequest {
            name: project_name.into(),
        });

        // ACT
        let _ = save_load_context.save_project(save_request_1).await;
        let _ = save_load_context.save_project(save_request_2).await;
        let response = save_load_context.load_project(load_request).await;
        let load_project_proto = response.unwrap().into_inner().project.unwrap();

        // ASSERT
        assert_eq!(load_project_proto, project_proto_2)
    }
}
