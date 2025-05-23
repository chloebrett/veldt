use super::{
    EnvelopeSelector, GeneratorEffectSelector, LfoSelector, ModMatrixCellSelector,
    OscillatorSelector, Selector, SelectorTrait,
};
use crate::StoreData;
use shared::model::GeneratorInstance;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct GeneratorSelector(/* generator_index */ pub usize);

impl GeneratorSelector {
    pub fn downcast_oscillator(&self, oscillator_index: usize) -> OscillatorSelector {
        OscillatorSelector(self.0, oscillator_index)
    }
    pub fn downcast_lfo(&self, lfo_index: usize) -> LfoSelector {
        LfoSelector(self.0, lfo_index)
    }
    pub fn downcast_envelope(&self, envelope_index: usize) -> EnvelopeSelector {
        EnvelopeSelector(self.0, envelope_index)
    }
    pub fn downcast_effect(&self, effect_index: usize) -> GeneratorEffectSelector {
        GeneratorEffectSelector(self.0, effect_index)
    }
    pub fn downcast_mod_matrix_cell(&self, row: usize, col: usize) -> ModMatrixCellSelector {
        ModMatrixCellSelector(self.0, row, col)
    }
}

impl SelectorTrait for GeneratorSelector {
    type Item = GeneratorInstance;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.generators.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.generators.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Generator(self.0)
    }
}
