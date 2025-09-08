use crate::Action;
use std::cmp::min;

mod effect;
mod generator;
mod matrix_cell;
mod mixer_channel;
mod note;
mod placement;
mod project;
mod sample;
mod store_data;
mod track;
mod drums;

/// A model object that can receive actions.
pub trait ActionReceiver {
    /// Tries to apply an action to a model object.
    /// If the action is applied, returns Some(action) where `action` reverses the change.
    /// If the action isn't relevant, returns None.
    fn apply(&mut self, action: &Action) -> Option<Action>;
}

/// Moves an element in a vector to a particular index.
/// Other elements move around it accordingly.
pub fn move_elem<T: Clone>(vec: &mut Vec<T>, from_index: usize, to_index: usize) {
    let elem = vec.remove(min(from_index, vec.len() - 1));
    vec.insert(min(to_index, vec.len()), elem);
}

pub fn delete_elems<T: Clone>(vec: &mut Vec<T>, indexes: Vec<usize>) {
    let mut indexes = indexes;
    indexes.sort();
    // Delete indexes in reverse so that indexes not yet removed are not changed during operation.
    for index in indexes.iter().rev() {
        vec.remove(*index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_elem_simple() {
        let mut v = vec![10, 11, 12, 13];

        move_elem(&mut v, 1, 3);

        // 11 (index 1) has moved to index 3.
        assert_eq!(v, vec![10, 12, 13, 11]);
    }
}
