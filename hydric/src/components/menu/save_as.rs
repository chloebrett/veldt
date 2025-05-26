use state::{Action, TypeField};

use crate::{
    view::View,
    widget::{StateWindow, default_window},
    window_state::{WindowKind, WindowState},
};
use egui::Ui;

pub struct SaveAs<'a, F: Fn(Action), G: FnMut()> {
    window_state: &'a WindowState,
    name: &'a String,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut()> SaveAs<'a, F, G> {
    pub fn new(window_state: &'a WindowState, name: &'a String, dispatch: F, on_click: G) -> Self {
        Self {
            window_state,
            name,
            dispatch,
            on_click,
        }
    }
}

impl<F: Fn(Action), G: FnMut()> View for SaveAs<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let SaveAs {
            window_state,
            name,
            dispatch,
            on_click,
        } = self;
        let window = StateWindow(
            default_window(name)
                .default_pos(window_state.get_pos(WindowKind::Save))
                .resizable(false),
        );
        window.show_with_closure(
            ui,
            window_state.get_visible(WindowKind::Save),
            |_| window_state.set_visible(WindowKind::Save, false),
            |ui| {
                let mut temp_name = name.clone();
                let response = ui.text_edit_singleline(&mut temp_name);
                if response.changed() {
                    dispatch(Action::SetChild(TypeField::ProjectName(temp_name)));
                }

                if ui.button("Save").clicked() {
                    on_click()
                }
            },
        );
    }
}
