use super::app::App;
use crate::rpc::{load_track, load_track_list, save_track};
use egui::Ui;
use poll_promise::Promise;

pub fn save_button(app: &mut App, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let track_name = app.store.project.name.clone();
        let track_to_save = app.store.project.tracks[0].clone();
        app.save_track_promise = Some(Promise::spawn_local(async move {
            save_track(track_name, track_to_save).await
        }));
        app.track_list_promise = Promise::spawn_local(async move { load_track_list().await });
    }
}

pub fn load_control(app: &mut App, ui: &mut Ui) {
    if let Some(list) = app.track_list_promise.ready() {
        app.track_list = list.clone().unwrap_or(vec![]);
    }
    if let Some(track_promise) = &app.track_promise {
        if let Some(track) = track_promise.ready() {
            app.store.project.tracks[0] = track.clone().unwrap();
            app.track_promise = None;
        }
    }
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(1) // TODO Correct Id Salt
            .selected_text(
                app.load_track_name
                    .clone()
                    .unwrap_or("".to_string())
                    .to_string(),
            )
            .show_ui(ui, |ui| {
                for name in app.track_list.iter() {
                    ui.selectable_value(&mut app.load_track_name, Some(name.clone()), name);
                }
            });
        // TODO disable button when no load_name
        if ui.button("Load").clicked() {
            let load_name = app.load_track_name.clone().unwrap();
            app.track_promise = Some(Promise::spawn_local(
                async move { load_track(load_name).await },
            ))
        };
    });
}
