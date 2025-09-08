use crate::view::View;
use crate::widget::StateWindow;
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState};
use egui::{Rect, Ui};
use shared::model::{PlacementId, PlacementType, SampleId};
use state::{DrumTrackSelector, Store};

pub struct DrumRackView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> DrumRackView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }

    fn drum_sub_track_ui(&self, ui: &mut Ui, sample_id: SampleId) {
        ui.label(format!("Placeholder for {:?}", *sample_id));
    }

    fn empty_drum_sub_track_ui(&self, ui: &mut Ui) {
        ui.label(format!("This is empty"));
    }
}

impl View for DrumRackView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Some(placement_id): Option<PlacementId> = self.local_state.active_placement.get()
        else {
            return;
        };
        let Some(drum_track_sel): Option<DrumTrackSelector> =
            self.local_state.active_drum_track.get()
        else {
            return;
        };
        let drum_track_id = drum_track_sel.0;
        self.local_state
            .window_state
            .set_visible(WindowKind::DrumRack, true);
        let title = format!("Drum Rack for Placement {:?}", *placement_id);
        let placement = self
            .store
            .get()
            .project
            .placements
            .get(&placement_id)
            .unwrap();
        if let PlacementType::DrumTrack(drum_track_placement) = &placement.kind {
            let drum_sub_tracks = drum_track_placement
                .drum_track
                .drum_sub_tracks
                .keys()
                .clone();
            StateWindow::show_from_window_state(
                ui,
                &self.local_state.window_state,
                WindowKind::DrumRack,
                &title,
                |ui| {    
                    let num_non_empty = drum_sub_tracks.len();                
                    for &sample_id in drum_sub_tracks {
                        self.drum_sub_track_ui(ui, sample_id);
                    }
                    let mut empty_drum_sub_tracks = self.local_state.empty_drum_sub_tracks.borrow_mut();
                    let num_empty_sub_tracks = empty_drum_sub_tracks.get(&drum_track_id).cloned().unwrap_or_else(|| {
                        let num_empty = (4-num_non_empty).max(0);
                        empty_drum_sub_tracks.insert(drum_track_id, num_empty);
                        num_empty
                    });

                    for _ in 0..num_empty_sub_tracks {
                        self.empty_drum_sub_track_ui(ui);
                    }
                    
                    let add_drums_btn = ui.button("Add Drums");
                    if add_drums_btn.clicked() {
                        // until the new drum sub track has a sample set for it only add the drum sub track to local state
                        empty_drum_sub_tracks.insert(drum_track_id, num_empty_sub_tracks + 1);
                    };
                },
            );
        }
    }
}
