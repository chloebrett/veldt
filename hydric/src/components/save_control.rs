use super::AsyncState;
use crate::promise::poll;
use crate::rpc::{load_project, load_project_list, save_project};
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
    poll(&mut async_state.save_project, /* if_ready= */ |_| {
        async_state.project_list = Some(Promise::spawn_local(
            async move { load_project_list().await },
        ));
    });

    poll(
        &mut async_state.project_list,
        /* if_ready= */
        |list: &Vec<String>| {
            store.dispatchr(Action::SetProjectList {
                projects: list.clone(),
            });
        },
    );

    poll(
        &mut async_state.load_project,
        /* if_ready= */
        |project: &Project| {
            store.dispatchr(Action::SetProject {
                project: project.clone(),
            });
        },
    );

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
                    Some(Promise::spawn_local(
                        async move { load_project(name).await },
                    ));
            }
        };
    });
}
