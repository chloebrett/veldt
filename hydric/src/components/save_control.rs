use super::app::App;
use crate::rpc::{load_note_list, load_notes, save_notes};
use egui::Ui;
use poll_promise::Promise;

pub fn save_button(app: &mut App, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let save_name = app.track_name.clone();
        let notes_to_save = app.notes.clone();
        app.save_notes_promise = Some(Promise::spawn_local(async move {
            save_notes(save_name, notes_to_save).await
        }));
        app.notes_list_promise = Promise::spawn_local(async move { load_note_list().await });
    }
}

pub fn load_control(app: &mut App, ui: &mut Ui) {
    if let Some(list) = app.notes_list_promise.ready() {
        app.track_list = list.clone().unwrap_or(vec![]);
    }
    if let Some(notes_promise) = &app.notes_promise {
        if let Some(notes) = notes_promise.ready() {
            if let Some(values) = notes {
                app.notes = values.to_vec();
                app.notes_promise = None
            }
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
        if ui.button("Load").clicked() {
            let load_name = app.load_track_name.clone().unwrap();
            app.notes_promise = Some(Promise::spawn_local(
                async move { load_notes(load_name).await },
            ))
        };
    });
}
