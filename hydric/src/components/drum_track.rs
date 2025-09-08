use crate::view::View;
use crate::widget::{StateWindow};
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState};
use egui::{Rect, Ui};
use shared::model::{DrumTrackId, PlacementId, PlacementType, Sample, SampleId};
use state::{DrumTrackSelector, Store};
use std::collections::HashSet;
use crate::widget::{selectable_value, get_set};
use state::{Action, TypeField};

pub struct DrumTrackView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> DrumTrackView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }

    fn drum_sub_track_ui(&self, ui: &mut Ui, sample_id: SampleId) {
        ui.label(format!("Placeholder for {:?}", *sample_id));
    }

    fn empty_drum_sub_track_ui(&self, ui: &mut Ui, drum_track_id: &DrumTrackId, index: usize) {
        ui.horizontal(|ui| {

            let available_samples = self.get_available_samples(drum_track_id);
            let sel = DrumTrackSelector(*drum_track_id);

            egui::ComboBox::from_id_salt(format!("sub_track_{:?}_{}", drum_track_id, index))
                .selected_text("Choose a sample")
                .show_ui(ui, |ui| {
                    for sample_id in available_samples {
                        let sample_option = ui.selectable_label(false, self.store.get().project.samples.get(&sample_id).unwrap().sample_name.clone());
                        if sample_option.clicked() {
                            // store doesn't track empty subtracks, so once sample is selected the subtrack gets added not set
                            // subtrack is considered empty if it doesn't have a sample associated with it
                            self.store.dispatch(&sel, Action::AddChild(TypeField::SampleId(sample_id)));
                        }
                    }
                });
        });

        ui.separator();
    }

    fn get_available_samples(&self, drum_track_id: &DrumTrackId) -> Vec<SampleId> {
        let all_samples: HashSet<SampleId> = self.store.get().project.samples.keys().cloned().collect();
        let taken_samples: HashSet<SampleId>= self.store.get().project.drum_tracks.get(drum_track_id).unwrap().drum_sub_tracks.keys().cloned().collect();
        let mut available_samples: Vec<SampleId> = all_samples
            .difference(&taken_samples)
            .cloned()
            .collect();
        available_samples.sort_by_key(|sample_id| *sample_id);
        available_samples
    }
}

impl View for DrumTrackView<'_> {
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
        let title = format!("Drum Track {:?}", *drum_track_sel.0);
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

                    const DEFAULT_NUM_EMPTY: usize = 4;
                    let mut empty_drum_sub_tracks = self.local_state.empty_drum_sub_tracks.borrow_mut();
                    let num_empty_sub_tracks = empty_drum_sub_tracks.get(&drum_track_id).cloned().unwrap_or_else(|| {
                        let num_empty = (DEFAULT_NUM_EMPTY-num_non_empty).max(0);
                        empty_drum_sub_tracks.insert(drum_track_id, num_empty);
                        num_empty
                    });

                    for index in 0..num_empty_sub_tracks {
                        self.empty_drum_sub_track_ui(ui, &drum_track_id, index);
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
