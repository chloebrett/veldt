use state::{Action, TypeField};

use crate::{local_state::LocalState, view::View, widget::StateWindow, window_state::WindowKind};
use egui::Ui;

pub struct SaveAs<'a, F: Fn(Action), G: FnMut()> {
    local_state: &'a LocalState,
    name: &'a String,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut()> SaveAs<'a, F, G> {
    pub fn new(local_state: &'a LocalState, name: &'a String, dispatch: F, on_click: G) -> Self {
        Self {
            local_state,
            name,
            dispatch,
            on_click,
        }
    }
}

impl<F: Fn(Action), G: FnMut()> View for SaveAs<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let SaveAs {
            local_state,
            name,
            dispatch,
            on_click,
        } = self;
        StateWindow::show_from_window_state(
            ui,
            &local_state.window_state,
            WindowKind::Save,
            name,
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
