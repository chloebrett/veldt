use crate::widget::{get_set, inner_frame_dark, selectable_value, slider};
use crate::components::effect::channel_name;
use egui::Ui;
use mesic::samples_to_beats;
use ordered_float::OrderedFloat;
use shared::model::{Placement, PlacementId, SampleId, SamplePlacement};
use shared::types::Beats;
use state::{Action, PlacementSelector, SampleSelector, Store, TypeField, IndexField};
use std::cmp::max;

pub struct SamplePlacementView<'a> {
    store: &'a Store,
}

impl<'a> SamplePlacementView<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn ui(
        &self,
        ui: &mut Ui,
        placement_id: PlacementId,
        placement: &Placement,
        sample_placement: &SamplePlacement,
        sel: &PlacementSelector,
    ) {
        ui.vertical(|ui| {
            let samples_exist = !self.store.get().project.samples.is_empty();
            let selected_text = if samples_exist {
                self.get_sample_name(&sample_placement.sample_id)
            } else {
                "No samples loaded yet".to_string()
            };

            egui::ComboBox::from_id_salt(format!("placement_{:?}", placement_id))
                .width(215.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    if samples_exist {
                        for sample_id in self.store.get().project.samples.keys() {
                            selectable_value(
                                ui,
                                get_set(&sample_placement.sample_id, |it| {
                                    self.store
                                        .dispatch(sel, Action::SetChild(TypeField::SampleId(*it)))
                                }),
                                sample_id,
                                self.get_sample_name(sample_id),
                            );
                        }
                    } else {
                        ui.label("No samples to select");
                    }
                });
        });

        ui.add_space(5.0);
        egui::ComboBox::from_id_salt(format!("placement_{:?}_channel", placement_id))
            .selected_text(channel_name(sample_placement.mixer_channel))
            .show_ui(ui, |ui| {
                for channel in 0..self.store.get().project.mixer.channels.len() {
                    selectable_value(
                        ui,
                        get_set(sample_placement.mixer_channel, |it| {
                            self.store
                                .dispatch(sel, Action::SetIndex(IndexField::Mixer(it)))
                        }),
                        channel,
                        channel_name(channel),
                    );
                }
            });

        ui.add_space(5.0);
        let sample_sel = SampleSelector(sample_placement.sample_id);
        let Some(sample) = self.store.try_select(&sample_sel) else {
            return;
        };

        let max_duration = samples_to_beats(
            max(sample.left.len(), sample.right.len()),
            self.store.get().project.bpm,
        );
        let duration = *placement
            .clipped_duration
            .unwrap_or(OrderedFloat(max_duration));
        self.duration_ui(ui, sel, duration, max_duration);
    }

    fn get_sample_name(&self, sample_id: &SampleId) -> String {
        let mut sample_name = format!("Sample ID {}", **sample_id);
        if let Some(sample) = self.store.get().project.samples.get(sample_id) {
            sample_name = sample.sample_name.clone();
        }
        if self.store.get().project.samples.is_empty() {
            sample_name = "".to_string();
        }
        sample_name
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
