use crate::view::View;
use crate::widget::{default_window, get_set, selectable_value, slider};
use egui::{Id, Ui, pos2};
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
        let window_id = Id::new("track_placement_window");
        let window_state =
            ui.data_mut(|data| *data.get_temp_mut_or_insert_with(window_id, move || false));
        let placement_id = Id::new("active_track_placement_index");
        let active_index = ui.data_mut(|data| {
            *data.get_temp_mut_or_insert_with::<Option<usize>>(placement_id, move || None)
        });
        // Return if there is no active track_placement or window is not open
        let placement_index = if let Some(index) = active_index {
            if window_state { index } else { return }
        } else {
            return;
        };
        let on_release = || store.dispatchr(Action::Release);
        let placement = &store.get().project.track_placements[placement_index];
        let tracks_length = store.get().project.tracks.len();
        let sel = Selector::TrackPlacement(placement_index);
        let mut open = window_state;
        default_window(&format!("Track Placement {}", placement_index))
            .open(&mut open)
            .default_pos(pos2(100.0, 20.0))
            .resizable(true)
            .show(ui.ctx(), |ui| {
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
                    log::info!("!");
                    ui.data_mut(|data| {
                        data.insert_temp(window_id, false);
                        data.insert_temp::<Option<usize>>(placement_id, None);
                    });
                }
            });
        if window_state != open {
            ui.data_mut(|data| {
                data.insert_temp(window_id, false);
            })
        }
    }
}
