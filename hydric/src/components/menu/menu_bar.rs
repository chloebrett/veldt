use crate::{
    AsyncState,
    local_state::LocalState,
    playback::AudioPlayer,
    promise::{poll, spawn},
    rpc::{export, load_project, load_project_list, save_project},
    view::View,
    window_state::WindowKind,
};
use egui::{Button, Ui, menu::bar};
use state::{Action, Store, TypeField};

use super::generator::GeneratorMenuOptions;
use super::{effect::EffectMenuOptions, save_as::SaveAs};

pub struct MenuBar<'a> {
    store: &'a mut Store,
    local_state: &'a LocalState,
    player: &'a mut AudioPlayer,
    async_state: &'a mut AsyncState,
}

impl<'a> MenuBar<'a> {
    pub fn new(
        store: &'a mut Store,
        local_state: &'a LocalState,
        player: &'a mut AudioPlayer,
        async_state: &'a mut AsyncState,
    ) -> Self {
        Self {
            store,
            local_state,
            player,
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
        SaveAs::new(self.local_state, name, dispatch, save_click).ui(ui);
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
                    self.local_state
                        .window_state
                        .set_visible(WindowKind::Save, true);
                }
                ui.menu_button("Load", |ui| {
                    self.load_options(ui);
                });

                ui.menu_button("Export", |ui| {
                    if ui.button("WAV").clicked() {
                        let project = self.store.get().project.clone();
                        spawn(&mut self.async_state.export, async move {
                            export(project, "wav".to_string()).await
                        });
                    }
                    if ui.button("MP3").clicked() {
                        let project = self.store.get().project.clone();
                        spawn(&mut self.async_state.export, async move {
                            export(project, "mp3".to_string()).await
                        });
                    }
                });
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
                let mut window_button = |label, window_kind: WindowKind| {
                    let state = self.local_state.window_state.get_visible(window_kind);
                    if ui.add(Button::new(label).selected(state)).clicked() {
                        self.local_state
                            .window_state
                            .set_visible(window_kind, !state)
                    }
                };
                window_button("Mixer", WindowKind::Mixer);
                window_button("Generators", WindowKind::GeneratorList);
                window_button("Scale", WindowKind::Scale);
                window_button("Samples", WindowKind::SampleTree);
                window_button("Track Roll", WindowKind::TrackRoll);
                window_button("Microphone", WindowKind::Microphone);
                ui.separator();
                if ui.button("Close all".to_string()).clicked() {
                    self.local_state.window_state.close_all();
                };
            });
            ui.menu_button("Effects", |ui| {
                EffectMenuOptions::new(self.store, self.local_state).ui(ui);
            });
            // TODO: create an "add generator" dropdown similar to the effects one.
            ui.menu_button("Debug", |ui| {
                if ui.button("Recreate mixer").clicked() {
                    self.player.refresh_mixer();
                }
                if ui.button("Init mixer graph debug").clicked() {
                    self.player.init_mixer_debug();
                }
                if ui.button("Show mixer graph debug").clicked() {
                    self.local_state
                        .window_state
                        .set_visible(WindowKind::GraphDebug, true);
                }
            });
            let mut window_icon = |icon, label, window_kind| {
                let state = self.local_state.window_state.get_visible(window_kind);
                let icon_response = ui.add(Button::new(icon).selected(state));
                if icon_response.clicked() {
                    self.local_state
                        .window_state
                        .set_visible(window_kind, !state);
                }
                icon_response.on_hover_ui(|ui| {
                    ui.label(label);
                });
            };
            window_icon("📂", "Samples", WindowKind::SampleTree);
            window_icon("🎷", "Generators", WindowKind::GeneratorList);
            window_icon("🎨", "Mixer", WindowKind::Mixer);
            window_icon("📄", "Track Roll", WindowKind::TrackRoll);
            window_icon("🎤", "Record Microphone", WindowKind::Microphone);

            ui.menu_button("Generators", |ui| {
                GeneratorMenuOptions::new(self.store, self.local_state).ui(ui);
            });
        });
    }
}
