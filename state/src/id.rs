use shared::model::{EffectId, EffectInstance};

use crate::{Store, TypeField};

/// An abstraction for handling fields on the Project stored by ID in the store.
pub trait Id<T> {
    /// Receive the next ID for a project field.
    fn next_id(store: &Store) -> T;
    /// Wrap the item in the respective TypeField variant.
    fn get_child(&self) -> TypeField;
    /// Wrap the ID in the respective TypeField ID variant.
    fn get_id_type_field(id: &T) -> TypeField;
}

impl Id<EffectId> for EffectInstance {
    fn next_id(store: &Store) -> EffectId {
        store
            .get()
            .project
            .effects
            .clone()
            .into_keys()
            .max()
            .map(|EffectId(it)| EffectId(it + 1))
            .unwrap_or(EffectId(0))
    }

    fn get_child(&self) -> TypeField {
        TypeField::Effect(self.clone())
    }

    fn get_id_type_field(id: &EffectId) -> TypeField {
        TypeField::EffectId(*id)
    }
}
