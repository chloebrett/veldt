use crate::view::View;
use crate::widget::StateWindow;
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState};
use egui::Ui;
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
}

impl View for DrumRackView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Some(placement_id): Option<PlacementId> = self.local_state.active_placement.get()
        else {
            return;
        };
        let Some(_drum_track_sel): Option<DrumTrackSelector> =
            self.local_state.active_drum_track.get()
        else {
            return;
        };
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
                    ui.heading("PLACEHOLDER");
                    for sample_id in drum_sub_tracks {
                        self.drum_sub_track_ui(ui, *sample_id);
                    }
                },
            );
        }
    }
}
