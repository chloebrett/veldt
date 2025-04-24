use state::{Action, TypeField};

use crate::{view::View, widget::{default_window, get_set, string_observer}};
use egui::Ui;

pub struct SaveAs<'a, F: Fn(Action), G: FnMut() > {
    open: bool,
    name: &'a String,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut()> SaveAs<'a, F, G> {
    pub fn new(open: bool, name: &'a String, dispatch: F, on_click: G) -> Self {
        SaveAs {
            open, name,dispatch,on_click
        }
    }
}

impl<'a, F: Fn(Action), G: FnMut()> View for SaveAs<'a, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let SaveAs {
            open, name, dispatch, on_click
        } = self;
        default_window("Save Project").open(open).show(ui.ctx(), |ui| {
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
    }
}
