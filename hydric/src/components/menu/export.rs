use state::{Action, TypeField};

use crate::{
    app_state::WindowState,
    view::View,
    widget::{default_window, get_set, string_observer},
};
use egui::{Ui, pos2};

pub struct Export<'a, F: Fn(Action), G: FnMut()> {
    window_state: &'a mut WindowState,
    name: &'a String,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut()> Export<'a, F, G> {
    pub fn new(
        window_state: &'a mut WindowState,
        name: &'a String,
        dispatch: F,
        on_click: G,
    ) -> Self {
        Export {
            window_state,
            name,
            dispatch,
            on_click,
        }
    }
}

impl<F: Fn(Action), G: FnMut()> View for Export<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Export {
            window_state,
            name,
            dispatch,
            on_click,
        } = self;

        let screen_size = ui.ctx().used_size();
        default_window("Export to .wav")
            .default_pos(pos2(screen_size.x / 2.0, screen_size.y / 2.0))
            .open(&mut window_state.export) // use a dedicated export window toggle
            .show(ui.ctx(), |ui| {
                let mut name_observer = string_observer(
                    get_set(name.clone(), |it| {
                        dispatch(Action::SetChild(TypeField::ProjectName(it)))
                    }),
                    name.clone(),
                );
                ui.text_edit_singleline(&mut name_observer);

                if ui.button("Export").clicked() {
                    on_click()
                }
            });
    }
}
