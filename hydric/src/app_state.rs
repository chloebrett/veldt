use crate::{audio_player::Handle, promise::AsyncResult};
use egui::{Id, Ui};
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

// Interact with UI level state.
// State is stored in a `IdTypeMap` on `Ui` (native to egui) and so requires `&mut` access to `Ui`.
// Only store superficial, cheap to clone state variables that are local to a User and required to
// render the UI.
// E.g.
//   - Current selected track
//   - If an editor window is open.
pub enum DataState {
    ActiveTrackIndex,
    ActiveTrackPlacementIndex,
    ActiveNoteIndex,
    TrackPlacementViewWindow,
    NoteRollWindow,
    NoteWindow,
}

impl DataState {
    fn get_id(&self) -> Id {
        Id::new(match self {
            Self::ActiveNoteIndex => "active_note_index",
            Self::ActiveTrackPlacementIndex => "active_track_placement_index",
            Self::ActiveTrackIndex => "active_track_index",
            Self::TrackPlacementViewWindow => "track_placement_window",
            Self::NoteRollWindow => "note_roll_window",
            Self::NoteWindow => "note_window",
        })
    }

    pub fn get_value<T: 'static + Clone + Send + Sync>(&self, ui: &mut Ui) -> Option<T> {
        ui.data_mut(|data| {
            data.get_temp_mut_or_insert_with::<Option<T>>(self.get_id(), move || None)
                .clone()
        })
    }

    pub fn set_value<T: 'static + Clone + Send + Sync>(&self, ui: &mut Ui, value: T) {
        ui.data_mut(|data| {
            data.insert_temp(self.get_id(), Some(value));
        })
    }

    pub fn remove_value(&self, ui: &mut Ui) {
        ui.data_mut(|data| {
            // Value are stores by (Id, type) and so type of the value when not `None` must be
            // known.
            match self {
                Self::TrackPlacementViewWindow | Self::NoteWindow | Self::NoteRollWindow => {
                    data.insert_temp::<Option<bool>>(self.get_id(), None)
                }
                Self::ActiveNoteIndex
                | Self::ActiveTrackIndex
                | Self::ActiveTrackPlacementIndex => {
                    data.insert_temp::<Option<usize>>(self.get_id(), None)
                }
            };
        })
    }
}
