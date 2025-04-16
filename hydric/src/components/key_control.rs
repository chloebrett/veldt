use crate::view::View;
use crate::widget::{get_set, selectable_value};
use egui::{ComboBox, Ui};
use shared::model::{Scale, ScaleValue};
use state::Action;
use strum::IntoEnumIterator;

pub struct KeyControl<F: Fn(Action)> {
    dispatch: F,
    key: ScaleValue,
    scale: Scale,
}

impl<F: Fn(Action)> KeyControl<F> {
    pub fn new(dispatch: F, key: ScaleValue, scale: Scale) -> Self {
        KeyControl {
            dispatch,
            key,
            scale,
        }
    }
}

impl<F: Fn(Action)> View for KeyControl<F> {
    fn ui(&self, ui: &mut Ui) {
        let KeyControl {
            ref dispatch,
            key,
            scale,
        } = *self;
        ComboBox::from_label("Key")
            .selected_text(key.to_string())
            .show_ui(ui, |ui| {
                for scale_note in ScaleValue::iter() {
                    selectable_value(
                        ui,
                        get_set(key, |it| dispatch(Action::SetKey(it))),
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
                        get_set(scale, |it| dispatch(Action::SetScale(it))),
                        scale_option,
                        scale_option.to_string(),
                    );
                }
            });
    }
}
