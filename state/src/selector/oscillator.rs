use super::{GeneratorSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::{GeneratorId, Oscillator, StingrayConfig};

impl OscillatorSelector {
    pub fn upcast(&self) -> GeneratorSelector {
        GeneratorSelector(self.0)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct OscillatorSelector(pub GeneratorId, /* oscillator_index */ pub usize);

impl SelectorTrait for OscillatorSelector {
    type Item = Oscillator;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let instance = store.project.generators.get(&self.0)?;
        let stingray: &StingrayConfig = (&instance.it).try_into().ok()?;
        stingray.oscillators.get(self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let instance = store.project.generators.get_mut(&self.0)?;
        let stingray: &mut StingrayConfig = (&mut instance.it).try_into().ok()?;
        stingray.oscillators.get_mut(self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::Oscillator(self.0, self.1)
    }
}
