use super::AsyncState;
use crate::promise::{poll, spawn};
use crate::rpc::{load_project, load_project_list, save_project};
use crate::widget::selectable_value;
use egui::Ui;
use state::{Action, Store, get_set};

pub fn save_button(store: &Store, async_state: &mut AsyncState, ui: &mut Ui) {
    if ui.button("Save").clicked() {
        let project_name = store.get().project.name.clone();
        let project = store.get().project.clone();
        spawn(&mut async_state.save_project, async move {
            save_project(project_name, project).await
        })
    }
}

pub fn load_control(store: &Store, async_state: &mut AsyncState, ui: &mut Ui) {
    poll(&mut async_state.save_project, |_| {
        spawn(&mut async_state.project_list, async move {
            load_project_list().await
        });
    });

    poll(&mut async_state.project_list, |list| {
        store.dispatchr(Action::SetProjectList(list.clone()));
    });

    poll(&mut async_state.load_project, |project| {
        store.dispatchr(Action::SetProject(project.clone()));
    });

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
                                store.dispatchr(Action::SetLoadProjectName(it));
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
                spawn(&mut async_state.load_project, async move {
                    load_project(name).await
                })
            }
        };
    });
}
