use crate::promise::AsyncResult;
use dasp_frame::Stereo;
use shared::model::{FilenameTree, Project, Sample};

/// Container for the various promises launchable by the app.
#[derive(Default)]
pub struct AsyncState {
    pub server_render: AsyncResult<Vec<Stereo<f32>>, ()>,
    pub save_project: AsyncResult<(), ()>,
    pub project_list: AsyncResult<Vec<String>, ()>,
    pub load_project: AsyncResult<Project, ()>,
    pub load_sample: AsyncResult<Sample, ()>,
    pub upload_sample: AsyncResult<(), ()>,
    pub load_sample_tree: AsyncResult<FilenameTree, ()>,
    pub export: AsyncResult<(), ()>,
    pub microphone_start: AsyncResult<(), String>,
    pub microphone_stop:AsyncResult<Vec<u8>, String>,
}
