use crate::components::effect::channel_name;
use crate::widget::{get_set, selectable_value};
use egui::Ui;
use shared::model::{DrumTrackPlacement, PlacementId, SampleId, TrackId};
use state::{Action, IndexField, PlacementSelector, Store, TypeField};

pub struct DrumPlacementView<'a> {
    store: &'a Store,
}

impl<'a> DrumPlacementView<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn ui(
        &self,
        ui: &mut Ui,
        placement_id: PlacementId,
        drum_placement: &DrumTrackPlacement,
        sel: &PlacementSelector,
    ) {
        let track_sel = state::TrackSelector(drum_placement.track_id);
        let Some(_drum_track) = self.store.try_select(&track_sel) else {
            ui.label("No drum tracks added yet");
            return;
        };

        ui.horizontal(|ui| {
            ui.set_width(185.0);

            egui::ComboBox::from_id_salt(format!("placement_{:?}", placement_id))
                .selected_text(self.get_drum_track_name(&drum_placement.track_id))
                .show_ui(ui, |ui| {
                    for track_id in self.store.get().project.tracks.keys() {
                        selectable_value(
                            ui,
                            get_set(&drum_placement.track_id, |it| {
                                self.store
                                    .dispatch(sel, Action::SetChild(TypeField::TrackId(*it)))
                            }),
                            track_id,
                            self.get_drum_track_name(track_id),
                        );
                    }
                });

            ui.add_space(5.0);
            egui::ComboBox::from_id_salt(format!("placement_{:?}_channel", placement_id))
                .selected_text(channel_name(drum_placement.mixer_channel))
                .show_ui(ui, |ui| {
                    for channel in 0..self.store.get().project.mixer.channels.len() {
                        selectable_value(
                            ui,
                            get_set(drum_placement.mixer_channel, |it| {
                                self.store
                                    .dispatch(sel, Action::SetIndex(IndexField::Mixer(it)))
                            }),
                            channel,
                            channel_name(channel),
                        );
                    }
                });
        });
        ui.add_space(5.0);
    }

    fn get_drum_track_name(&self, track_id: &TrackId) -> String {
        if self.store.get().project.tracks.is_empty() {
            "".to_string()
        } else {
            format!("Track ID {}", **track_id)
        }
    }
}

pub fn get_sample_name(store: &Store, sample_id: &SampleId) -> String {
    let mut sample_name = format!("Sample ID {}", **sample_id);
    if let Some(sample) = store.get().project.samples.get(sample_id) {
        sample_name = sample.sample_name.clone();
    }
    if store.get().project.samples.is_empty() {
        sample_name = "".to_string();
    }
    sample_name
}
