use egui::{Id, Ui};
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
            };
        })
    }
}
