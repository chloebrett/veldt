use super::{NoteSelector, Selector, SelectorTrait};
use crate::StoreData;
use shared::model::Track;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct TrackSelector(/* track_index */ pub usize);

impl TrackSelector {
    pub fn downcast_note(&self, note_index: usize) -> NoteSelector {
        NoteSelector(self.0, note_index)
    }
}

impl SelectorTrait for TrackSelector {
    type Item = Track;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.tracks.get(self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.tracks.get_mut(self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::Track(self.0)
    }
}
