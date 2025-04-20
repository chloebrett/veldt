use log::info;
use prost::Message;
use shared::pmodel::ProjectProto;
use shared::save_load::{
    LoadProjectListReply, LoadProjectListRequest, LoadProjectReply, LoadProjectRequest,
    SaveProjectReply, SaveProjectRequest, save_load_server::SaveLoad,
};
use std::env::current_dir;
use std::fs::File;
use std::fs::{ReadDir, read_dir};
use std::io::{Read, Write};
use std::path::PathBuf;
use tonic::async_trait;

// Stateless: we just save projects to files and don't keep anything in memory.
pub struct SaveLoadContext;

fn project_dir_path() -> PathBuf {
    let mut file_path = current_dir().unwrap();
    file_path.pop(); // pop '/xeric'
    file_path.push("assets");
    file_path.push("projects");
    file_path
}

fn project_file_path(filename: String) -> PathBuf {
    let mut file_path = project_dir_path();
    // TODO: add a file extension.
    file_path.push(filename.clone());
    file_path
}

#[async_trait]
impl SaveLoad for SaveLoadContext {
    /// Saves a project to server memory by name.
    async fn save_project(
        self: &Self,
        request: tonic::Request<SaveProjectRequest>,
    ) -> Result<tonic::Response<SaveProjectReply>, tonic::Status> {
        let SaveProjectRequest { name, project } = request.into_inner();
        let mut project_bytes = vec![];

        // TODO: handle error.
        let _ = project.clone().unwrap().encode(&mut project_bytes);

        let file_path = project_file_path(name.clone());
        info!("Saving project to path: {}", file_path.clone().display());

        // Note: if the file already exists, it will be overwritten.
        let mut file = File::create(file_path)?;
        file.write_all(&project_bytes)?;

        info!("Saved {}", name.clone());
        info!("Saved project: {:?}", project.clone().unwrap());
        Ok(tonic::Response::new(SaveProjectReply {}))
    }

    /// Loads the list of project names that are saved. LoadProject can then be called to load an
    /// actual project.
    async fn load_project_list(
        self: &Self,
        _request: tonic::Request<LoadProjectListRequest>,
    ) -> Result<tonic::Response<LoadProjectListReply>, tonic::Status> {
        let dir_contents: ReadDir = read_dir(project_dir_path())?;
        let mut list: Vec<String> = vec![];

        for file in dir_contents {
            let file = file?;
            let filename: String = file
                .path()
                .iter()
                .next_back()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            list.push(filename);
        }

        Ok(tonic::Response::new(LoadProjectListReply {
            project_names: list,
        }))
    }

    /// Loads a project by name. If it doesn't exist in the server memory, returns an error.
    async fn load_project(
        self: &Self,
        request: tonic::Request<LoadProjectRequest>,
    ) -> Result<tonic::Response<LoadProjectReply>, tonic::Status> {
        let name = request.into_inner().name;

        let file_path = project_file_path(name.clone());
        info!("Loading project from path: {}", file_path.clone().display());

        let mut file = File::open(file_path)?;
        let mut project_buffer = vec![];
        file.read_to_end(&mut project_buffer)?;

        // TODO: handle error.
        let project = ProjectProto::decode(project_buffer.as_slice()).unwrap();
        info!("Loaded {} {}", name, project_buffer.len());
        info!("Loaded project: {:?}", project.clone());

        Ok(tonic::Response::new(LoadProjectReply {
            project: Some(project),
        }))
    }
}
