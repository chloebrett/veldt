use crate::{
    app_state::{AsyncState, WindowState},
    promise::{poll, spawn},
    rpc::{load_project, load_project_list, save_project},
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
        MenuBar {
            store,
            window_state,
            async_state,
        }
    }
}

fn save(
    ui: &mut Ui,
    store: &mut Store,
    window_state: &mut WindowState,
    async_state: &mut AsyncState,
) {
    let dispatch = |action: Action| store.dispatchr(action);

    poll(&mut async_state.save_project, |_| {
        spawn(&mut async_state.project_list, async move {
            load_project_list().await
        });
    });

    let save_click = || {
        let project = store.get().project.clone();
        spawn(&mut async_state.save_project, async move {
            save_project(project).await
        });
    };

    let name = &store.get().project.name;
    SaveAs::new(window_state, name, dispatch, save_click).ui(ui);
}

fn load_options(ui: &mut Ui, store: &Store, async_state: &mut AsyncState) {
    poll(&mut async_state.project_list, |list| {
        store.dispatchr(Action::SetChild(TypeField::ProjectList(list.clone())));
    });

    poll(&mut async_state.load_project, |project| {
        store.dispatchr(Action::SetChild(TypeField::Project(project.clone())));
    });

    let dispatch = |action| store.dispatchr(action);
    let mut load_click = |name: String| {
        spawn(&mut async_state.load_project, async move {
            load_project(name).await
        })
    };
    let project_names = &store.get().project_list;
    for name in project_names {
        let button_response = ui.add(Button::new(name).wrap_mode(egui::TextWrapMode::Extend));
        if button_response.clicked() {
            dispatch(Action::SetChild(TypeField::LoadProjectName(name.clone())));
            load_click(name.clone());
        }
    }
}

impl View for MenuBar<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let MenuBar {
            store,
            window_state,
            async_state,
        } = self;
        save(ui, store, window_state, async_state);
        bar(ui, |ui| {
            ui.label("Veldt");
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked() {
                    let project = store.get().project.clone();
                    spawn(&mut async_state.save_project, async move {
                        save_project(project).await
                    });
                }
                if ui.button("Save As").clicked() {
                    window_state.save = true;
                }
                ui.menu_button("Load", |ui| {
                    load_options(ui, store, async_state);
                });
                if ui.button("Export").clicked() {}
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(store.can_undo(), Button::new("Undo"))
                    .clicked()
                {
                    store.pend_undo();
                }
                if ui
                    .add_enabled(store.can_redo(), Button::new("Redo"))
                    .clicked()
                {
                    store.pend_redo();
                }
            });
            ui.menu_button("Windows", |ui| {
                let mut button_with_tick = |label, state: &mut bool| {
                    let suffix = if *state { " ✅" } else { "" };
                    if ui.button(format!("{}{}", label, suffix)).clicked() {
                        *state = !*state
                    }
                };
                button_with_tick("Mixers", &mut window_state.mixer.visible);
                button_with_tick("Generators", &mut window_state.generator_list);
                button_with_tick("Scale", &mut window_state.scale);
                button_with_tick("Samples", &mut window_state.sample_tree);
                button_with_tick("Track Roll", &mut window_state.track_roll);
            });
            ui.menu_button("Effects", |ui| if ui.button("Add effect").clicked() {});
            ui.menu_button(
                "Generators",
                |ui| {
                    if ui.button("Add generator").clicked() {}
                },
            );
            let sample_response = ui.add(Button::new("📂").selected(window_state.sample_tree));
            if sample_response.clicked() {
                window_state.sample_tree ^= true;
            }
            sample_response.on_hover_ui(|ui| {
                ui.label("Samples");
            });
            let sound_response = ui.add(Button::new("🎷"));
            sound_response.on_hover_ui(|ui| {
                ui.label("Sound library");
            });
            let effect_response = ui.add(Button::new("🎨").selected(window_state.mixer.visible));
            if effect_response.clicked() {
                window_state.mixer.visible ^= true;
            }
            effect_response.on_hover_ui(|ui| {
                ui.label("Effects/Mixers");
            });
            let track_response = ui.add(Button::new("📄").selected(window_state.track_roll));
            if track_response.clicked() {
                window_state.track_roll ^= true;
            }
            track_response.on_hover_ui(|ui| {
                ui.label("Track Roll");
            })
        });
    }
}
