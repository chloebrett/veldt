use crate::{audio_player::AudioPlayer, promise::AsyncResult};
use egui::{Id, Pos2, Ui};
use dasp_frame::Stereo;
use shared::model::{FilenameTree, Project, Sample};
use std::cmp::{Eq, Ord};
use std::collections::HashSet;
use std::hash::Hash;

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
}

#[derive(Default)]
pub struct AudioState {
    pub audio: Vec<Stereo<f32>>,
    pub player: AudioPlayer,
}

pub struct MixerWindowState {
    pub visible: bool,
    // Currently active / shown channel.
    pub channel: usize,
}

pub type EffectSelector = (usize, usize);
pub type GeneratorSelector = usize;

/// Which windows are currently shown.
pub struct WindowState {
    pub mixer: MixerWindowState,
    pub effects: WindowStateField<EffectSelector>,
    pub generator_list: bool,
    pub generators: WindowStateField<GeneratorSelector>,
    pub scale: bool,
    pub sample_tree: bool,
    pub track_roll: bool,
    pub save: bool,
}

impl Default for WindowState {
    fn default() -> WindowState {
        WindowState {
            mixer: MixerWindowState {
                visible: false,
                channel: 0,
            },
            effects: WindowStateField(HashSet::new()),
            generator_list: false,
            generators: WindowStateField(HashSet::new()),
            scale: false,
            sample_tree: false,
            track_roll: false,
            save: false,
        }
    }
}

#[derive(Clone)]
pub struct WindowStateField<T: Hash + Eq + Copy>(HashSet<T>);

impl<T: Hash + Ord + Copy> WindowStateField<T> {
    pub fn get(&self, index: T) -> bool {
        self.0.contains(&index)
    }

    pub fn set(&mut self, index: T, visible: bool) {
        let was_visible = self.get(index);
        if visible == was_visible {
            return;
        }
        if visible {
            self.0.insert(index);
        } else {
            self.0.retain(|it| *it != index);
        }
    }

    // Note: not necessarily sorted.
    pub fn as_vec(self) -> Vec<T> {
        self.0.into_iter().collect()
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
    SelectedNoteIndexes,
    SelectedTrackPlacementIndexes,
    DragCursorDelta,
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
            Self::SelectedNoteIndexes => "selected_note_indexes",
            Self::SelectedTrackPlacementIndexes => "selected_track_placement_indexes",
            Self::DragCursorDelta => "drag_start_from",
        })
    }

    pub fn get_value<T: 'static + Clone + Send + Sync>(&self, ui: &Ui) -> Option<T> {
        ui.data_mut(|data| {
            data.get_temp_mut_or_insert_with::<Option<T>>(self.get_id(), move || None)
                .clone()
        })
    }

    pub fn set_value<T: 'static + Clone + Send + Sync>(&self, ui: &Ui, value: T) {
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
                Self::SelectedNoteIndexes | Self::SelectedTrackPlacementIndexes => {
                    data.insert_temp::<Option<HashSet<usize>>>(self.get_id(), None);
                }
                Self::DragCursorDelta => data.insert_temp::<Option<Pos2>>(self.get_id(), None),
            };
        })
    }
}

/// Update the state of selected Sequencer Objects based on Ui interaction.
pub fn update_select_data_state(ui: &mut Ui, data_state: DataState, index: Option<usize>) {
    if let Some(it) = index {
        if let Some(mut selected) = data_state.get_value::<HashSet<usize>>(ui) {
            // If index is already in the set remove it.
            if selected.contains(&it) {
                selected.remove(&it);
            } else {
                selected.insert(it);
            }
            data_state.set_value(ui, selected)
        } else {
            data_state.set_value::<HashSet<usize>>(ui, HashSet::from_iter(vec![it]))
        }
    } else {
        // If there was no index supplied, remove value.
        data_state.remove_value(ui);
    }
}
