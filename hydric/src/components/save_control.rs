use super::app::App;
use super::envelope_control;
use crate::audio_player::{Handle, play};
use crate::rpc::render;
use crate::rpc::{load_note_list, load_notes, save_notes};
use egui::{
    Color32, Rect, ScrollArea, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    scroll_area::ScrollBarVisibility, vec2,
};
use mesic::{SAMPLE_RATE, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use strum::IntoEnumIterator;

pub fn save_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let save_name = app.track_name.clone();
        let notes_to_save = app.notes.clone();
        app.save_notes_promise = Some(Promise::spawn_local(async move {
            save_notes(save_name, notes_to_save).await
        }));
        app.notes_list_promise = Promise::spawn_local(async move { load_note_list().await });
    }

    if let Some(list) = app.notes_list_promise.ready() {
        match list {
            Some(values) => app.track_list = values.to_vec(),
            None => app.track_list = vec![],
        }
    }
    egui::ComboBox::from_label("Saved Tracks")
        .selected_text(app.track_name.clone())
        .show_ui(ui, |ui| {
            for name in app.track_list.iter() {
                ui.selectable_value(&mut app.track_name, name.clone(), name);
            }
        });
    if ui.button("Load").clicked() {
        let load_name = app.track_name.clone();
        app.notes_promise = Some(Promise::spawn_local(
            async move { load_notes(load_name).await },
        ))
    }
    if let Some(notes_promise) = &app.notes_promise {
        if let Some(notes) = notes_promise.ready() {
            match notes {
                Some(values) => app.notes = values.to_vec(),
                None => {}
            }
        }
    }
}
