use crate::AsyncState;
use crate::promise::{poll, spawn};
use crate::rpc::{load_project, load_project_list, save_project};
use crate::view::View;
use crate::widget::{get_set, selectable_value, string_observer};
use egui::Ui;
use state::{Action, Store, TypeField};

pub struct SaveLoadView<'a> {
    store: &'a Store,
    async_state: &'a mut AsyncState,
}

impl<'a> SaveLoadView<'a> {
    pub fn new(store: &'a Store, async_state: &'a mut AsyncState) -> Self {
        SaveLoadView { store, async_state }
    }
}

impl View for SaveLoadView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let project_name = self.store.get().project.name.clone();
        let mut name_observer = string_observer(
            get_set(project_name.clone(), |it| {
                self.store
                    .dispatchr(Action::SetChild(TypeField::ProjectName(it)))
            }),
            project_name.clone(),
        );
        ui.text_edit_singleline(&mut name_observer);

        if ui.button("Save").clicked() {
            let project = self.store.get().project.clone();
            spawn(&mut self.async_state.save_project, async move {
                save_project(project).await
            })
        }

        poll(&mut self.async_state.save_project, |_| {
            spawn(&mut self.async_state.project_list, async move {
                load_project_list().await
            });
        });

        poll(&mut self.async_state.project_list, |list| {
            self.store
                .dispatchr(Action::SetChild(TypeField::ProjectList(list.clone())));
        });

        poll(&mut self.async_state.load_project, |project| {
            self.store
                .dispatchr(Action::SetChild(TypeField::Project(project.clone())));
        });

        let load_project_name = self.store.get().load_project_name.clone();
        let project_list = &self.store.get().project_list;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(1) // TODO Correct Id Salt
                .selected_text(
                    load_project_name
                        .clone()
                        .unwrap_or("".to_string())
                        .to_string(),
                )
                .show_ui(ui, |ui| {
                    for name in project_list.iter() {
                        selectable_value(
                            ui,
                            get_set(load_project_name.clone(), |it| {
                                if let Some(it) = it {
                                    self.store.dispatchr(Action::SetChild(
                                        TypeField::LoadProjectName(it),
                                    ));
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
                    spawn(&mut self.async_state.load_project, async move {
                        load_project(name).await
                    })
                }
            };
        });
    }
}
