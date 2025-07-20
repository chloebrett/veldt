use shared::model::{EffectId, EffectInstance};

use crate::Store;

/// An abstraction for handling fields on the Project stored by ID in the store.
pub trait Id<T> {
    /// Receive the next ID for a project field.
    fn next_id(store: &Store) -> T;
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
}
