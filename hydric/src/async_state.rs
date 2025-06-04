use crate::promise::AsyncResult;
use dasp_frame::Stereo;
use shared::model::{FilenameTree, Project, Sample};

/// Container for the various promises launchable by the app.
#[derive(Default)]
pub struct AsyncState {
    pub server_render: AsyncResult<Vec<Stereo<f32>>, tonic::Status>,
    pub save_project: AsyncResult<(), tonic::Status>,
    pub project_list: AsyncResult<Vec<String>, tonic::Status>,
    pub load_project: AsyncResult<Project, tonic::Status>,
    pub load_sample: AsyncResult<Sample, tonic::Status>,
    pub upload_sample: AsyncResult<String, tonic::Status>,
    pub load_sample_tree: AsyncResult<FilenameTree, tonic::Status>,
    pub export: AsyncResult<(), tonic::Status>,
}
