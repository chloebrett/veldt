use state::{Action, TypeField};

use crate::{
    WindowState,
    view::View,
    widget::{default_window, get_set, string_observer},
};
use egui::{Ui, pos2};

pub struct SaveAs<'a, F: Fn(Action), G: FnMut()> {
    window_state: &'a mut WindowState,
    name: &'a String,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut()> SaveAs<'a, F, G> {
    pub fn new(
        window_state: &'a mut WindowState,
        name: &'a String,
        dispatch: F,
        on_click: G,
    ) -> Self {
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
        let screen_size = ui.ctx().used_size();
        default_window("Save Project As")
            .default_pos(pos2(screen_size.x / 2.0, screen_size.y / 2.0))
            .open(&mut window_state.save)
            .show(ui.ctx(), |ui| {
                let mut temp_name = name.clone();
                let response = ui.text_edit_singleline(&mut temp_name);

                if response.changed() {
                    dispatch(Action::SetChild(TypeField::ProjectName(temp_name)));
                }

                if ui.button("Save").clicked() {
                    on_click()
                }
            });
    }
}
