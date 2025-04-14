use crate::widget::{get_set, selectable_value, slider};
use egui::Ui;
use ordered_float::OrderedFloat;
use shared::model::{TrackId, TrackPlacement};
use shared::types::Beats;
use state::{Action, Selector, Store};

pub fn track_placement_control(store: &Store, ui: &mut Ui) {
    let project = &store.get().project;
    let on_release = || store.dispatchr(Action::Release);

    for track_placement_index in 0..project.track_placements.len() {
        let placement = &project.track_placements[track_placement_index];
        let sel = Selector::TrackPlacement(track_placement_index);

        egui::ComboBox::from_id_salt(format!("track_placement_{track_placement_index}"))
            .selected_text(format!("Track {}", placement.track_id))
            .show_ui(ui, |ui| {
                for track_index in 0..project.tracks.len() {
                    selectable_value(
                        ui,
                        get_set(&track_index, |it| {
                            store.dispatch(&sel, Action::SetTrackPlacementTrackId(*it))
                        }),
                        &track_index,
                        track_index.to_string(),
                    );
                }
            });

        let offset = *placement.offset as f64;
        slider(
            ui,
            "Start position",
            offset,
            |it| store.dispatch(&sel, Action::SetTrackPlacementOffset(it as Beats)),
            0.0..=16.0,
            on_release,
        );

        let max_note_length =
            *store.get().project.tracks[placement.track_id].find_max_note_length();
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
                        Action::SetTrackPlacementClippedDuration(clipped_duration)
                    })
                },
                0.0..=max_note_length as f64,
                on_release,
            );
        });

        if store.get().project.track_placements.len() > 1 && ui.button("Delete").clicked() {
            store.dispatchr(Action::DeleteTrackPlacement(track_placement_index));
            break;
        }
    }

    if ui.button("New track placement").clicked() {
        store.dispatchr(Action::AddTrackPlacement(TrackPlacement {
            track_id: 0 as TrackId,
            offset: OrderedFloat(0.0 as Beats),
            clipped_duration: None,
            visual_placement: 0,
        }));
    }
}
