use super::{Action, Store};

pub fn reducer(store: &mut Store, action: Action) {
    match action {
        Action::SetKey(key) => store.key = key,
        Action::SetScale(scale) => store.scale = scale,
        Action::SetProjectName(name) => store.project.name = name,
    }
}
