use egui::{Id, Pos2, Ui};
use std::collections::BTreeSet;

// Interact with UI level state.
// State is stored in a `IdTypeMap` on `Ui` (native to egui) and so requires `&mut` access to `Ui`.
// Only store superficial, cheap to clone state variables that are local to a User and required to
// render the UI.
// E.g.
//   - Current selected track
//   - If an editor window is open.
// TODO: rename TrackPlacement... to Placement here where appropriate.
pub enum DataState {
    ActiveTrackPlacementIndex,
    TrackPlacementViewWindow,
    SelectedTrackPlacementIndexes,
    DragCursorDelta,
    TrackRollSelectMode,
    NoteRollSelectMode,
    SubSynthLfoTab,
}

impl DataState {
    fn get_id(&self) -> Id {
        Id::new(match self {
            Self::ActiveTrackPlacementIndex => "active_track_placement_index",
            Self::TrackPlacementViewWindow => "track_placement_window",
            Self::SelectedTrackPlacementIndexes => "selected_track_placement_indexes",
            Self::DragCursorDelta => "drag_start_from",
            Self::TrackRollSelectMode => "track_roll_select_mode",
            Self::NoteRollSelectMode => "note_roll_select_mode",
            Self::SubSynthLfoTab => "subsynth_lfo_tab_index",
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
                Self::TrackPlacementViewWindow
                | Self::TrackRollSelectMode
                | Self::NoteRollSelectMode => data.insert_temp::<Option<bool>>(self.get_id(), None),
                Self::ActiveTrackPlacementIndex | Self::SubSynthLfoTab => {
                    data.insert_temp::<Option<usize>>(self.get_id(), None)
                }
                Self::SelectedTrackPlacementIndexes => {
                    data.insert_temp::<Option<BTreeSet<usize>>>(self.get_id(), None);
                }
                Self::DragCursorDelta => data.insert_temp::<Option<Pos2>>(self.get_id(), None),
            };
        })
    }
}

/// Update the state of selected Sequencer Objects based on Ui interaction.
pub fn update_select_data_state(ui: &mut Ui, data_state: DataState, index: Option<usize>) {
    if let Some(it) = index {
        if let Some(mut selected) = data_state.get_value::<BTreeSet<usize>>(ui) {
            // If index is already in the set remove it.
            if selected.contains(&it) {
                selected.remove(&it);
            } else {
                selected.insert(it);
            }
            data_state.set_value(ui, selected)
        } else {
            data_state.set_value::<BTreeSet<usize>>(ui, BTreeSet::from_iter(vec![it]))
        }
    } else {
        // If there was no index supplied, remove value.
        data_state.remove_value(ui);
    }
}
