use super::{Selector, SelectorTrait, TrackSelector};
use crate::StoreData;
use shared::model::{PlacedNote, TrackId};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct NoteSelector(
    pub TrackId,
    /* note_index */ pub usize,
);

impl NoteSelector {
    pub fn upcast(&self) -> TrackSelector {
        TrackSelector(self.0)
    }
}

impl SelectorTrait for NoteSelector {
    type Item = PlacedNote;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store
            .project
            .tracks
            .get(&self.0)
            .and_then(|it| it.notes.get(self.1))
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store
            .project
            .tracks
            .get_mut(&self.0)
            .and_then(|it| it.notes.get_mut(self.1))
    }

    fn as_enum(&self) -> Selector {
        Selector::Note(self.0, self.1)
    }
}
