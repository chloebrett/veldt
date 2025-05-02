use crate::DataState;
use crate::view::View;
use crate::widget::{StateWindow, default_window, get_set, int_slider, selectable_value, slider};
use egui::{Ui, pos2};
use ordered_float::OrderedFloat;
use shared::model::TrackPlacement;
use shared::types::Beats;
use state::{Action, FloatField, IndexField, PlacementSelector, Store, TypeField};

// TODO: rename to PlacementView if appropriate.
pub struct TrackPlacementView<'a> {
    store: &'a Store,
}

impl<'a> TrackPlacementView<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
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
        let placement = &store.get().project.placements[placement_index];
        let track_placement: &TrackPlacement = placement.try_into().unwrap();
        let tracks_length = store.get().project.tracks.len();
        let sel = PlacementSelector(placement_index);
        let title = format!("Placement {placement_index}");

        let window = StateWindow(
            default_window(&title)
                .default_pos(pos2(100.0, 20.0))
                .resizable(true),
        );
        window.show(ui, DataState::TrackPlacementViewWindow, |ui| {
            egui::ComboBox::from_id_salt(format!("placement_{placement_index}"))
                .selected_text(format!("Track {}", track_placement.track_index))
                .show_ui(ui, |ui| {
                    for track_index in 0..tracks_length {
                        selectable_value(
                            ui,
                            get_set(&track_placement.track_index, |it| {
                                store.dispatch(&sel, Action::SetIndex(IndexField::Track(*it)))
                            }),
                            &track_index,
                            track_index.to_string(),
                        );
                    }
                });

            // TODO: better UI than a slider for this!
            let max_generator_index = (store.get().project.generators.len() - 1) as i32;
            int_slider(
                ui,
                "Generator index",
                track_placement.generator_index as f64,
                |it| store.dispatch(&sel, Action::SetIndex(IndexField::Generator(it as usize))),
                0..=max_generator_index,
                on_release,
            );

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
                *store.get().project.tracks[track_placement.track_index].unclipped_duration();
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
                store.dispatchr(Action::DeleteChild(IndexField::Placement(placement_index)));
                DataState::TrackPlacementViewWindow.set_value(ui, false);
                DataState::ActiveTrackPlacementIndex.remove_value(ui);
            }
        });
    }
}
