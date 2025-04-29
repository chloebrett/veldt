use crate::view::View;
use crate::widget::{default_window, get_set, selectable_value};
use egui::{ComboBox, Pos2, Ui};
use shared::model::{Scale, ScaleValue};
use state::{Action, TypeField};
use strum::IntoEnumIterator;

pub struct KeyView<'a, F: Fn(Action)> {
    dispatch: F,
    visible: &'a mut bool,
    key: ScaleValue,
    scale: Scale,
}

impl<'a, F: Fn(Action)> KeyView<'a, F> {
    pub fn new(dispatch: F, visible: &'a mut bool, key: ScaleValue, scale: Scale) -> Self {
        Self {
            dispatch,
            visible,
            key,
            scale,
        }
    }
}

impl<F: Fn(Action)> View for KeyView<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            ref dispatch,
            key,
            scale,
            ..
        } = *self;

        let visible = &mut self.visible;

        default_window("Scale")
            .open(visible)
            .default_pos(Pos2 { x: 600.0, y: 20.0 })
            .show(ui.ctx(), |ui| {
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
                                get_set(scale, |it| {
                                    dispatch(Action::SetChild(TypeField::Scale(it)))
                                }),
                                scale_option,
                                scale_option.to_string(),
                            );
                        }
                    });
            });
    }
}
