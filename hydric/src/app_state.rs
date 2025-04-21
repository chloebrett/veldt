use crate::{audio_player::Handle, promise::AsyncResult};
use shared::model::{FilenameTree, Project, Sample};

/// Container for the various promises launchable by the app.
#[derive(Default)]
pub struct AsyncState {
    pub server_render: AsyncResult<Vec<f32>, ()>,
    pub save_project: AsyncResult<(), ()>,
    pub project_list: AsyncResult<Vec<String>, ()>,
    pub load_project: AsyncResult<Project, ()>,
    pub load_sample: AsyncResult<Sample, ()>,
    pub load_sample_tree: AsyncResult<FilenameTree, ()>,
}

#[derive(Default)]
pub struct AudioState {
    pub audio: Vec<f32>,
    pub handle: Option<Handle>,
    pub pre_render: bool,
}

pub struct MixerWindowState {
    pub visible: bool,
    // Currently active / shown channel.
    pub channel: usize,
}

/// Which windows are currently shown.
pub struct WindowState {
    pub mixer: MixerWindowState,
    pub effects: Vec<Vec<bool>>, // by ID (within each mixer)
    pub generator_list: bool,
    pub generators: Vec<bool>, // by ID
    pub scale: bool,
    pub sample_tree: bool,
    pub track_roll: bool,
}

impl Default for WindowState {
    fn default() -> WindowState {
        // TODO: generate this automatically from the project state.
        WindowState {
            mixer: MixerWindowState {
                visible: false,
                channel: 0,
            },
            effects: vec![vec![false, false, false, false]],
            generator_list: false,
            generators: vec![true, true],
            scale: false,
            sample_tree: false,
            track_roll: false,
        }
    }
}
