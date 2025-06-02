use super::{GeneratorSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::{LfoConfig, StingrayConfig, GeneratorId};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct LfoSelector(
    pub GeneratorId,
    /* lfo_index */ pub usize,
);

impl LfoSelector {
    pub fn upcast(&self) -> GeneratorSelector {
        GeneratorSelector(self.0)
    }
}

impl SelectorTrait for LfoSelector {
    type Item = LfoConfig;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let instance = store.project.generators.get(&self.0)?;
        let stingray: &StingrayConfig = (&instance.it).try_into().ok()?;
        stingray.lfos.get(self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let instance = store.project.generators.get_mut(&self.0)?;
        let stingray: &mut StingrayConfig = (&mut instance.it).try_into().ok()?;
        stingray.lfos.get_mut(self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::Lfo(self.0, self.1)
    }
}
