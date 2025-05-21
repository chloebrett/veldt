use super::{Selector, SelectorTrait};
use crate::StoreData;
use shared::model::Sample;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct SampleSelector(/* sample_index */ pub usize);

impl SelectorTrait for SampleSelector {
    type Item = Sample;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.samples.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.samples.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Sample(self.0)
    }
}
