use crate::view::View;
use crate::widget::{get_set, selectable_value};
use egui::{ComboBox, Ui};
use shared::model::{Scale, ScaleValue};
use state::{Action, TypeField};
use strum::IntoEnumIterator;

pub struct KeyView<F: Fn(Action)> {
    dispatch: F,
    key: ScaleValue,
    scale: Scale,
}

impl<F: Fn(Action)> KeyView<F> {
    pub fn new(dispatch: F, key: ScaleValue, scale: Scale) -> Self {
        KeyView {
            dispatch,
            key,
            scale,
        }
    }
}

impl<F: Fn(Action)> View for KeyView<F> {
    fn ui(&mut self, ui: &mut Ui) {
        let KeyView {
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
                        get_set(key, |it| dispatch(Action::SetChild(TypeField::Key(it)))),
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
                        get_set(scale, |it| dispatch(Action::SetChild(TypeField::Scale(it)))),
                        scale_option,
                        scale_option.to_string(),
                    );
                }
            });
    }
}
