use state::{Action, TypeField};

use crate::{app_state::WindowState ,view::View, widget::{default_window, get_set, string_observer}};
use egui::Ui;

pub struct SaveAs<'a, F: Fn(Action), G: FnMut() > {
    window_state: &'a mut WindowState,
    name: &'a String,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut()> SaveAs<'a, F, G> {
    pub fn new(window_state: &'a mut WindowState, name: &'a String, dispatch: F, on_click: G) -> Self {
        SaveAs {
            window_state, name,dispatch,on_click
        }
    }
}

impl<'a, F: Fn(Action), G: FnMut()> View for SaveAs<'a, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let SaveAs {
            window_state, name, dispatch, on_click
        } = self;
        let mut open = window_state.save;
        default_window("Save Project As...").open(&mut open).show(ui.ctx(), |ui| {
            let mut name_observer = string_observer(
                get_set(name.clone(), |it| {
                    dispatch(Action::SetChild(TypeField::ProjectName(it)))
                }),
                name.clone(),
            );
            ui.text_edit_singleline(&mut name_observer);

            if ui.button("Save").clicked() {
                on_click()
            }
        });
        if !open {
            window_state.save = false
        }
    }
}
