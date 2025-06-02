use super::{GeneratorSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::{AdsrEnvelope, GeneratorId, StingrayConfig};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct EnvelopeSelector(pub GeneratorId, /* envelope_index */ pub usize);

impl EnvelopeSelector {
    pub fn upcast(&self) -> GeneratorSelector {
        GeneratorSelector(self.0)
    }
}

impl SelectorTrait for EnvelopeSelector {
    type Item = AdsrEnvelope;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let instance = store.project.generators.get(&self.0)?;
        let stingray: &StingrayConfig = (&instance.it).try_into().ok()?;
        stingray.envelopes.get(self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let instance = store.project.generators.get_mut(&self.0)?;
        let stingray: &mut StingrayConfig = (&mut instance.it).try_into().ok()?;
        stingray.envelopes.get_mut(self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::Envelope(self.0, self.1)
    }
}
