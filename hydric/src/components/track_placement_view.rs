use crate::app_state::DataState;
use crate::view::View;
use crate::widget::{default_window, get_set, selectable_value, slider, window_state_show};
use egui::{Ui, pos2};
use ordered_float::OrderedFloat;
use shared::types::Beats;
use state::{Action, FloatField, IndexField, Selector, Store, TypeField, UintField};

pub struct TrackPlacementView<'a> {
    store: &'a Store,
}

impl<'a> TrackPlacementView<'a> {
    pub fn new(store: &'a Store) -> Self {
        TrackPlacementView { store }
    }
}

impl View for TrackPlacementView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = &mut self.store;
        let Some(placement_index): Option<usize> =
            DataState::ActiveTrackPlacementIndex.get_value(ui)
        else {
            return;
        };
        let on_release = || store.dispatchr(Action::Release);
        let placement = &store.get().project.track_placements[placement_index];
        let tracks_length = store.get().project.tracks.len();
        let sel = Selector::TrackPlacement(placement_index);
        let title = format!("Track Placement {placement_index}");
        let ctx = &ui.ctx().clone();
        let window = default_window(&title)
            .default_pos(pos2(100.0, 20.0))
            .resizable(true);
        window_state_show(ui, DataState::TrackPlacementViewWindow, window, ctx, |ui| {
            egui::ComboBox::from_id_salt(format!("track_placement_{placement_index}"))
                .selected_text(format!("Track {}", placement.track_id))
                .show_ui(ui, |ui| {
                    for track_index in 0..tracks_length {
                        selectable_value(
                            ui,
                            get_set(&placement.track_id, |it| {
                                store.dispatch(&sel, Action::SetUint(UintField::TrackId, *it))
                            }),
                            &(track_index as u32),
                            track_index.to_string(),
                        );
                    }
                });

            let offset = *placement.offset as f64;
            slider(
                ui,
                "Start position",
                offset,
                |it| store.dispatch(&sel, Action::SetFloat(FloatField::Offset, it as Beats)),
                0.0..=16.0,
                on_release,
            );

            let max_note_length =
                *store.get().project.tracks[placement.track_id as usize].unclipped_duration();
            let duration = *placement
                .clipped_duration
                .unwrap_or(OrderedFloat(max_note_length)) as f64;
            ui.horizontal(|ui| {
                slider(
                    ui,
                    "Clipped Duration",
                    duration,
                    |it| {
                        store.dispatch(&sel, {
                            let clipped_duration = if it < max_note_length as f64 {
                                Some(it as Beats)
                            } else {
                                None
                            };
                            Action::SetChild(TypeField::ClippedDuration(clipped_duration))
                        })
                    },
                    0.0..=max_note_length as f64,
                    on_release,
                );
            });

            if ui.button("Delete").clicked() {
                store.dispatchr(Action::DeleteChild(IndexField::TrackPlacement(
                    placement_index,
                )));
                DataState::TrackPlacementViewWindow.set_value(ui, false);
                DataState::ActiveTrackPlacementIndex.remove_value(ui);
            }
        });
    }
}
