use crate::widget::{selectable_value, slider};
use egui::Ui;
use ordered_float::OrderedFloat;
use shared::model::{TrackId, TrackPlacement};
use shared::state::{Action, Selector};
use shared::types::Beats;
use state::{Store, get_set};

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

        // TODO: add slider + on/off for clipped duration.

        if ui.button("Delete").clicked() {
            store.dispatchr(Action::DeleteTrackPlacement {
                track_placement_index,
            });
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
