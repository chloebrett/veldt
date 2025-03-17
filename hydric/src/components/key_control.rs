use crate::state::get_set;
use crate::state::{Action, Store};
use crate::widget::selectable_value;
use egui::Ui;
use shared::model::{Scale, ScaleValue};
use strum::IntoEnumIterator;

pub fn key_control(store: &Store, ui: &mut Ui) {
    let key = store.get().key;
    egui::ComboBox::from_label("Key")
        .selected_text(key.to_string())
        .show_ui(ui, |ui| {
            for scale_note in ScaleValue::iter() {
                selectable_value(
                    ui,
                    get_set(key, |it| store.dispatch(Action::SetKey(it))),
                    scale_note,
                    scale_note.to_string(),
                );
            }
        });

    let scale = store.get().scale;
    egui::ComboBox::from_label("Scale")
        .selected_text(scale.to_string())
        .show_ui(ui, |ui| {
            for scale in Scale::iter() {
                selectable_value(
                    ui,
                    get_set(scale, |it| store.dispatch(Action::SetScale(it))),
                    scale,
                    scale.to_string(),
                );
            }
        });
}
