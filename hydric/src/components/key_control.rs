use super::app::App;
use egui::Ui;
use shared::model::{Scale, ScaleValue};
use strum::IntoEnumIterator;

pub fn key_control(app: &mut App, ui: &mut Ui) {
    egui::ComboBox::from_label("Key")
        .selected_text(app.key.to_string())
        .show_ui(ui, |ui| {
            for scale_note in ScaleValue::iter() {
                ui.selectable_value(&mut app.key, scale_note, scale_note.to_string());
            }
        });
    egui::ComboBox::from_label("Scale")
        .selected_text(app.scale.to_string())
        .show_ui(ui, |ui| {
            for scale in Scale::iter() {
                ui.selectable_value(&mut app.scale, scale, scale.to_string());
            }
        });
}
