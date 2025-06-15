use super::{Selector, SelectorTrait};
use crate::StoreData;
use shared::model::{EffectId, EffectInstance};

#[derive(Default, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct EffectSelector(pub EffectId);

impl SelectorTrait for EffectSelector {
    type Item = EffectInstance;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.effects.get(&self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.effects.get_mut(&self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Effect(self.0)
    }
}
