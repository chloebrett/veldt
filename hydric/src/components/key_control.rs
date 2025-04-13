use crate::view::View;
use crate::widget::{get_set, selectable_value};
use egui::{ComboBox, Ui};
use shared::model::{Scale, ScaleValue};
use state::{Action, Store};
use strum::IntoEnumIterator;

pub struct KeyControl {
    key: ScaleValue,
    scale: Scale,
}

impl KeyControl {
    pub fn new(key: ScaleValue, scale: Scale) -> Self {
        KeyControl { key, scale }
    }
}

impl View for KeyControl {
    fn ui(&self, store: &Store, ui: &mut Ui) {
        let KeyControl { key, scale } = *self;
        ComboBox::from_label("Key")
            .selected_text(key.to_string())
            .show_ui(ui, |ui| {
                for scale_note in ScaleValue::iter() {
                    selectable_value(
                        ui,
                        get_set(key, |it| store.dispatchr(Action::SetKey(it))),
                        scale_note,
                        scale_note.to_string(),
                    );
                }
            });
        ComboBox::from_label("Scale")
            .selected_text(scale.to_string())
            .show_ui(ui, |ui| {
                for scale_option in Scale::iter() {
                    selectable_value(
                        ui,
                        get_set(scale, |it| store.dispatchr(Action::SetScale(it))),
                        scale_option,
                        scale_option.to_string(),
                    );
                }
            });
    }
}
