use crate::{
    AsyncState, WindowState,
    promise::{poll, spawn},
    rpc::{export, load_project, load_project_list, save_project},
    view::View,
};
use egui::{Button, Ui, menu::bar};
use state::{Action, Store, TypeField};

use super::save_as::SaveAs;

pub struct MenuBar<'a> {
    store: &'a mut Store,
    window_state: &'a mut WindowState,
    async_state: &'a mut AsyncState,
}

impl<'a> MenuBar<'a> {
    pub fn new(
        store: &'a mut Store,
        window_state: &'a mut WindowState,
        async_state: &'a mut AsyncState,
    ) -> Self {
        Self {
            store,
            window_state,
            async_state,
        }
    }

    fn save(&mut self, ui: &mut Ui) {
        let dispatch = |action: Action| self.store.dispatchr(action);

        poll(&mut self.async_state.save_project, |_| {
            spawn(&mut self.async_state.project_list, async move {
                load_project_list().await
            });
        });

        let save_click = || {
            let project = self.store.get().project.clone();
            spawn(&mut self.async_state.save_project, async move {
                save_project(project).await
            });
        };

        let name = &self.store.get().project.name;
        SaveAs::new(self.window_state, name, dispatch, save_click).ui(ui);
    }

    fn load_options(&mut self, ui: &mut Ui) {
        poll(&mut self.async_state.project_list, |list| {
            self.store
                .dispatchr(Action::SetChild(TypeField::ProjectList(list.clone())));
        });

        poll(&mut self.async_state.load_project, |project| {
            self.store
                .dispatchr(Action::SetChild(TypeField::Project(project.clone())));
        });

        let dispatch = |action| self.store.dispatchr(action);
        let mut load_click = |name: String| {
            spawn(&mut self.async_state.load_project, async move {
                load_project(name).await
            })
        };
        let project_names = &self.store.get().project_list;
        for name in project_names {
            let button_response = ui.add(Button::new(name).wrap_mode(egui::TextWrapMode::Extend));
            if button_response.clicked() {
                dispatch(Action::SetChild(TypeField::LoadProjectName(name.clone())));
                load_click(name.clone());
            }
        }
    }
}

impl View for MenuBar<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        self.save(ui);
        bar(ui, |ui| {
            ui.label("Veldt");
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked() {
                    let project = self.store.get().project.clone();
                    spawn(&mut self.async_state.save_project, async move {
                        save_project(project).await
                    });
                }
                if ui.button("Save As").clicked() {
                    self.window_state.save = true;
                }
                ui.menu_button("Load", |ui| {
                    self.load_options(ui);
                });

                if ui.button("Export").clicked() {
                    let project = self.store.get().project.clone();
                    spawn(&mut self.async_state.export, async move {
                        export(project).await
                    });
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(self.store.can_undo(), Button::new("Undo"))
                    .clicked()
                {
                    self.store.pend_undo();
                }
                if ui
                    .add_enabled(self.store.can_redo(), Button::new("Redo"))
                    .clicked()
                {
                    self.store.pend_redo();
                }
            });
            ui.menu_button("Windows", |ui| {
                let mut button_with_tick = |label, state: &mut bool| {
                    let suffix = if *state { " ✅" } else { "" };
                    if ui.button(format!("{}{}", label, suffix)).clicked() {
                        *state = !*state
                    }
                };
                button_with_tick("Mixers", &mut self.window_state.mixer.visible);
                button_with_tick("Generators", &mut self.window_state.generator_list);
                button_with_tick("Scale", &mut self.window_state.scale);
                button_with_tick("Samples", &mut self.window_state.sample_tree);
                button_with_tick("Track Roll", &mut self.window_state.track_roll);
            });
            ui.menu_button("Effects", |ui| if ui.button("Add effect").clicked() {});
            ui.menu_button(
                "Generators",
                |ui| {
                    if ui.button("Add generator").clicked() {}
                },
            );
            let sample_response = ui.add(Button::new("📂").selected(self.window_state.sample_tree));
            if sample_response.clicked() {
                self.window_state.sample_tree ^= true;
            }
            sample_response.on_hover_ui(|ui| {
                ui.label("Samples");
            });
            let sound_response = ui.add(Button::new("🎷"));
            sound_response.on_hover_ui(|ui| {
                ui.label("Sound library");
            });
            let effect_response =
                ui.add(Button::new("🎨").selected(self.window_state.mixer.visible));
            if effect_response.clicked() {
                self.window_state.mixer.visible ^= true;
            }
            effect_response.on_hover_ui(|ui| {
                ui.label("Effects/Mixers");
            });

            let track_response = ui.add(Button::new("📄").selected(self.window_state.track_roll));
            if track_response.clicked() {
                self.window_state.track_roll ^= true;
            }
            track_response.on_hover_ui(|ui| {
                ui.label("Track Roll");
            });
            let microphone_response =
                ui.add(Button::new("🎤").selected(self.window_state.microphone));
            if microphone_response.clicked() {
                self.window_state.microphone ^= true;
            }
            microphone_response.on_hover_ui(|ui| {
                ui.label("Record Microphone");
            })
        });
    }
}
