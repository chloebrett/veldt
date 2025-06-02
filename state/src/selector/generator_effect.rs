use super::{GeneratorSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::{EqConfig, StingrayConfig, GeneratorId};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct GeneratorEffectSelector(
    pub GeneratorId,
    /* effect_index */ pub usize,
);

impl GeneratorEffectSelector {
    pub fn upcast(&self) -> GeneratorSelector {
        GeneratorSelector(self.0)
    }
}

impl SelectorTrait for GeneratorEffectSelector {
    type Item = EqConfig;

    // NOTE: in theory, can support arbitrary generator effects,
    // but for now we only have stingray LPF
    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let instance = store.project.generators.get(&self.0)?;
        let stingray: &StingrayConfig = (&instance.it).try_into().ok()?;
        Some(&stingray.lpf)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let instance = store.project.generators.get_mut(&self.0)?;
        let stingray: &mut StingrayConfig = (&mut instance.it).try_into().ok()?;
        Some(&mut stingray.lpf)
    }

    fn as_enum(&self) -> Selector {
        Selector::GeneratorEffect(self.0, self.1)
    }
}
