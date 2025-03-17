use super::app::App;
use crate::state::Action;
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::{Scale, ScaleValue};
use strum::IntoEnumIterator;

pub fn key_control(app: &mut App, ui: &mut Ui) {
    egui::ComboBox::from_label("Key")
        .selected_text(app.store.get().key.to_string())
        .show_ui(ui, |ui| {
            for scale_note in ScaleValue::iter() {
                selectable_value(
                    ui,
                    |it| {
                        it.map(|it| app.store.dispatch(Action::SetKey(it)));
                        app.store.get().key
                    },
                    scale_note,
                    scale_note.to_string(),
                );
            }
        });
    egui::ComboBox::from_label("Scale")
        .selected_text(app.store.get().scale.to_string())
        .show_ui(ui, |ui| {
            for scale in Scale::iter() {
                selectable_value(
                    ui,
                    |it| {
                        it.map(|it| app.store.dispatch(Action::SetScale(it)));
                        app.store.get().scale
                    },
                    scale,
                    scale.to_string(),
                );
            }
        });
}
