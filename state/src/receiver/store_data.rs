use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, StoreData, TypeField};

impl ActionReceiver for StoreData {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.project.apply(action) {
            return Some(undo);
        }

        Some(match action {
            Action::SetChild(TypeField::Key(key)) => {
                let prev = self.key;
                self.key = *key;
                Action::SetChild(TypeField::Key(prev))
            }
            Action::SetChild(TypeField::Scale(scale)) => {
                let prev = self.scale;
                self.scale = *scale;
                Action::SetChild(TypeField::Scale(prev))
            }
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetChild(TypeField::ProjectList(projects)) => {
                self.project_list = projects.clone();
                Action::NonReversible
            }
            Action::SetChild(TypeField::Project(project)) => {
                self.project = project.clone();
                Action::NonReversible
            }
            Action::SetChild(TypeField::LoadProjectName(project_name)) => {
                self.load_project_name = Some(project_name.clone());
                Action::NonReversible
            }
            _ => return None,
        })
    }
}
