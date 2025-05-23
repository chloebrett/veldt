use super::{Selector, SelectorTrait};
use crate::StoreData;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct RootSelector;

impl SelectorTrait for RootSelector {
    type Item = StoreData;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        Some(store)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        Some(store)
    }

    fn as_enum(&self) -> Selector {
        Selector::Root
    }
}
