use super::AsyncState;
use crate::rpc::{load_project, save_project, load_project_list};
use crate::widget::selectable_value;
use egui::Ui;
use poll_promise::Promise;
use shared::model::Project;
use state::{Action, Store, get_set};

pub fn save_button(store: &Store, async_state: &mut AsyncState, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let project_name = store.get().project.name.clone();
        let project = store.get().project.clone();
        async_state.save_project = Some(Promise::spawn_local(async move {
            save_project(project_name, project).await
        }));
    }
}

pub fn load_control(store: &Store, async_state: &mut AsyncState, ui: &mut Ui) {
    {
        let save_project_promise: &Option<Promise<Option<()>>> = &async_state.save_project;
        let promise_ref: Option<&Promise<Option<()>>> = save_project_promise.as_ref();
        if let Some(promise) = promise_ref {
            if let Some(Some(())) = promise.ready() {
                async_state.save_project = None;

                // Once a project has been saved, re-load the list of project names;
                async_state.project_list =
                    Some(Promise::spawn_local(async move { load_project_list().await }));
            }
        }
    }

    {
        let project_list_promise: &Option<Promise<Option<Vec<String>>>> = &async_state.project_list;
        let promise_ref = project_list_promise.as_ref();
        let mut clear = false;
        if let Some(promise) = promise_ref {
            if let Some(Some(list)) = promise.ready() {
                clear = true;

                store.dispatchr(Action::SetProjectList {
                    projects: list.clone(),
                });
            }
        }
        if clear {
            async_state.project_list = None;
        }
    }

    // TODO: generalise this promise handling logic.
    {
        let load_project_promise: &Option<Promise<Option<Project>>> = &async_state.load_project;
        let promise_ref = load_project_promise.as_ref();
        let mut clear = false;
        if let Some(promise) = promise_ref {
            if let Some(Some(project)) = promise.ready() {
                clear = true;

                store.dispatchr(Action::SetProject {
                    project: project.clone(),
                });
            }
        }
        if clear {
            async_state.load_project = None;
        }
    }

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(1) // TODO Correct Id Salt
            .selected_text(
                store
                    .get()
                    .load_project_name
                    .clone()
                    .unwrap_or("".to_string())
                    .to_string(),
            )
            .show_ui(ui, |ui| {
                for name in store.get().project_list.iter() {
                    selectable_value(
                        ui,
                        get_set(store.get().load_project_name.clone(), |it| {
                            if let Some(it) = it {
                                store.dispatchr(Action::SetLoadProjectName { project_name: it });
                            }
                        }),
                        Some(name.clone()),
                        name,
                    );
                }
            });
        // TODO disable button when no load_name
        if ui.button("Load").clicked() {
            if let Some(name) = store.get().load_project_name.clone() {
                async_state.load_project =
                    Some(Promise::spawn_local(async move { load_project(name).await }));
            }
        };
    });
}
