use super::{MixerSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::EffectInstance;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct EffectSelector(
    /* mixer_index */ pub usize,
    /* effect_index */ pub usize,
);

impl EffectSelector {
    pub fn upcast(&self) -> MixerSelector {
        MixerSelector(self.0)
    }
}

impl SelectorTrait for EffectSelector {
    type Item = EffectInstance;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store
            .project
            .mixer
            .channels
            .get(self.0)
            .and_then(|it| it.effects.get(self.1))
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store
            .project
            .mixer
            .channels
            .get_mut(self.0)
            .and_then(|it| it.effects.get_mut(self.1))
    }

    fn as_enum(&self) -> Selector {
        Selector::Effect(self.0, self.1)
    }
}
