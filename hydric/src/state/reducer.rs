use super::{Action, StoreData};
use std::cell::RefMut;

pub fn reducer(mut data: RefMut<'_, StoreData>, action: Action) {
    match action {
        Action::SetKey(key) => data.key = key,
        Action::SetScale(scale) => data.scale = scale,
        Action::SetProjectName(name) => data.project.name = name,
    }
}
