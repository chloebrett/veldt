use crate::Action;

mod effect;
mod generator;
mod mixer_channel;
mod note;
mod placement;
mod project;
mod store_data;
mod track;

/// A model object that can receive actions.
pub trait ActionReceiver {
    /// Tries to apply an action to a model object.
    /// If the action is applied, returns Some(action) where `action` reverses the change.
    /// If the action isn't relevant, returns None.
    fn apply(&mut self, action: &Action) -> Option<Action>;
}

pub fn move_elem<T: Clone>(vec: &mut Vec<T>, from_index: usize, to_index: usize) {
    vec.insert(to_index, vec[from_index].clone());
    if from_index > to_index {
        // Inserted value has increased original index of value by 1
        vec.remove(from_index + 1);
    } else if from_index <= to_index {
        // Inserted value has not changed original index of value
        vec.remove(from_index);
    };
}

pub fn delete_elems<T: Clone>(vec: &mut Vec<T>, indexes: Vec<usize>) {
    let mut indexes = indexes;
    indexes.sort();
    // Delete indexes in reverse so that indexes not yet removed are not changed during operation.
    for index in indexes.iter().rev() {
        vec.remove(*index);
    }
}
