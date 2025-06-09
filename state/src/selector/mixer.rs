use super::{Selector, SelectorTrait};
use crate::StoreData;
use shared::model::MixerChannel;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct MixerSelector(/* mixer_index */ pub usize);

impl SelectorTrait for MixerSelector {
    type Item = MixerChannel;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.mixer.channels.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.mixer.channels.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Mixer(self.0)
    }
}
