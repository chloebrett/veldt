use state::{Action, TypeField};

use crate::{
    app_state::WindowState,
    view::View,
    widget::{default_window, get_set, selectable_value},
};
use egui::{Ui, pos2};

pub struct LoadView<'a, F: Fn(Action), G: FnMut(String)> {
    window_state: &'a mut WindowState,
    current_name: &'a Option<String>,
    project_names: &'a Vec<String>,
    dispatch: F,
    on_click: G,
}

impl<'a, F: Fn(Action), G: FnMut(String)> LoadView<'a, F, G> {
    pub fn new(
        window_state: &'a mut WindowState,
        current_name: &'a Option<String>,
        project_names: &'a Vec<String>,
        dispatch: F,
        on_click: G,
    ) -> Self {
        LoadView {
            window_state,
            current_name,
            project_names,
            dispatch,
            on_click,
        }
    }
}

impl<F: Fn(Action), G: FnMut(String)> View for LoadView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let LoadView {
            window_state,
            current_name,
            project_names,
            dispatch,
            on_click,
        } = self;
        let screen_size = ui.ctx().used_size();
        default_window("Load Project")
            .default_pos(pos2(screen_size.x / 2.0, screen_size.y / 2.0))
            .open(&mut window_state.load)
            .show(ui.ctx(), |ui| {
                let load_project_name = current_name.clone();
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(1) // TODO Correct Id Salt
                        .selected_text(
                            load_project_name
                                .clone()
                                .unwrap_or("".to_string())
                                .clone()
                        )
                        .show_ui(ui, |ui| {
                            for name in project_names.iter() {
                                selectable_value(
                                    ui,
                                    get_set(load_project_name.clone(), |it| {
                                        if let Some(it) = it {
                                            dispatch(Action::SetChild(TypeField::LoadProjectName(it)))

                                        }
                                    }),
                                    Some(name.clone()),
                                    name,
                                );
                            }
                        });
                    // TODO disable button when no load_name
                    if ui.button("Load").clicked() {
                        if let Some(name) = load_project_name.clone() {
                            on_click(name);
                        }
                    };
                });
            });
    }
}
