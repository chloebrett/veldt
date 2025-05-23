use super::{GeneratorSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::{MatrixCell, StingrayConfig};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct MixerMatrixCellSelector(/* row */ pub usize, /* col */ pub usize);

impl SelectorTrait for MixerMatrixCellSelector {
    type Item = MatrixCell;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.mixer.matrix.get(self.0, self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.mixer.matrix.get_mut(self.0, self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::MixerMatrixCell(self.0, self.1)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct ModMatrixCellSelector(
    /* generator_index */ pub usize,
    /* row */ pub usize,
    /* col */ pub usize,
);

impl ModMatrixCellSelector {
    pub fn upcast(&self) -> GeneratorSelector {
        GeneratorSelector(self.0)
    }
}

impl SelectorTrait for ModMatrixCellSelector {
    type Item = MatrixCell;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let instance = store.project.generators.get(self.0)?;
        let stingray: &StingrayConfig = (&instance.it).try_into().ok()?;
        stingray.matrix.get(self.1, self.2)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let instance = store.project.generators.get_mut(self.0)?;
        let stingray: &mut StingrayConfig = (&mut instance.it).try_into().ok()?;
        stingray.matrix.get_mut(self.1, self.2)
    }

    fn as_enum(&self) -> Selector {
        Selector::ModMatrixCell(self.0, self.1, self.2)
    }
}
