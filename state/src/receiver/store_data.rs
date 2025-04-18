use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, StoreData};

impl ActionReceiver for StoreData {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.project.apply(action) {
            return Some(undo);
        }

        Some(match action {
            Action::SetKey(key) => {
                let prev = self.key;
                self.key = *key;
                Action::SetKey(prev)
            }
            Action::SetScale(scale) => {
                let prev = self.scale;
                self.scale = *scale;
                Action::SetScale(prev)
            }
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetProjectList(projects) => {
                self.project_list = projects.clone();
                Action::NonReversible
            }
            Action::SetProject(project) => {
                self.project = project.clone();
                Action::NonReversible
            }
            Action::SetLoadProjectName(project_name) => {
                self.load_project_name = Some(project_name.clone());
                Action::NonReversible
            }
            _ => return None,
        })
    }
}
