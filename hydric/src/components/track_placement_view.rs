use crate::components::utils::ToEguiColour;
use crate::widget::{get_set, inner_frame_dark, selectable_value, slider};
use egui::Ui;
use ordered_float::OrderedFloat;
use shared::model::Track;
use shared::model::{GeneratorId, Placement, PlacementId, TrackId, TrackPlacement};
use shared::types::Beats;
use state::{Action, PlacementSelector, Store, TrackSelector, TypeField};
use std::cmp::max;

pub struct TrackPlacementView<'a> {
    store: &'a Store,
}

impl<'a> TrackPlacementView<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn ui(
        &self,
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement,
        track_placement: &TrackPlacement,
        sel: &PlacementSelector,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(180.0);

            egui::ComboBox::from_id_salt(format!("placement_{:?}_track", placement_id))
                .selected_text(format!("Track ID {}", *track_placement.track_id))
                .show_ui(ui, |ui| {
                    for track_id in self.store.get().project.tracks.keys() {
                        selectable_value(
                            ui,
                            get_set(&track_placement.track_id, |it| {
                                self.store
                                    .dispatch(sel, Action::SetChild(TypeField::TrackId(*it)))
                            }),
                            track_id,
                            track_id.to_string(),
                        );
                    }
                });

            egui::ComboBox::from_id_salt(format!("placement_{:?}_generator", placement_id))
                .selected_text(self.get_generator_name(&track_placement.generator_id))
                .show_ui(ui, |ui| {
                    let mut generators: Vec<_> =
                        self.store.get().project.generators.keys().collect();
                    generators.sort();
                    for generator_id in generators {
                        selectable_value(
                            ui,
                            get_set(&track_placement.generator_id, |it| {
                                self.store
                                    .dispatch(sel, Action::SetChild(TypeField::GeneratorId(*it)))
                            }),
                            generator_id,
                            self.get_generator_name(generator_id),
                        );
                    }
                });
        });

        let track_sel = TrackSelector(track_placement.track_id);
        let track: &Track = self.store.select(&track_sel);
        let max_duration = *(track.unclipped_duration());
        let duration = *placement
            .clipped_duration
            .unwrap_or(OrderedFloat(max_duration));
        ui.add_space(5.0);
        self.duration_ui(ui, sel, duration, max_duration);
    }

    fn get_generator_name(&self, generator_id: &GeneratorId) -> String {
        let mut generator_name = format!("Generator ID {}", **generator_id);
        if let Some(generator_instance) = self.store.get().project.generators.get(generator_id) {
            if !generator_instance.meta.name.is_empty() {
                generator_name = generator_instance.meta.name.clone();
            }
        }
        if self.store.get().project.generators.is_empty() {
            generator_name = "There are currently no generators".to_string();
        }
        generator_name
    }

    fn duration_ui(&self, ui: &mut Ui, sel: &PlacementSelector, duration: f32, max_duration: f32) {
        let on_release = || self.store.dispatchr(Action::Release);
        inner_frame_dark().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Clipped duration");
                    ui.add_space(95.0);
                });
                slider(
                    ui,
                    "",
                    duration as f64,
                    |it| {
                        self.store.dispatch(sel, {
                            let clipped_duration = if it < max_duration as f64 {
                                Some(it as Beats)
                            } else {
                                None
                            };
                            Action::SetChild(TypeField::ClippedDuration(clipped_duration))
                        })
                    },
                    0.0..=max_duration as f64,
                    on_release,
                );
            });
        });
    }
}
