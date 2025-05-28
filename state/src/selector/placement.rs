use super::{Selector, SelectorTrait};
use crate::StoreData;
use shared::model::Placement;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct PlacementSelector(/* placement_index */ pub usize);

impl SelectorTrait for PlacementSelector {
    type Item = Placement;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.placements.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.placements.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Placement(self.0)
    }
}
