use super::{Selector, SelectorTrait};
use crate::{DrumSubTrackSelector, StoreData};
use shared::model::{DrumTrack, DrumTrackId, SampleId};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct DrumTrackSelector(pub DrumTrackId);

impl DrumTrackSelector {
    pub fn downcast_drum_sub_track(&self, sample_id: SampleId) -> DrumSubTrackSelector {
        DrumSubTrackSelector(self.0, sample_id)
    }
}

impl SelectorTrait for DrumTrackSelector {
    type Item = DrumTrack;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        store.project.drum_tracks.get(&self.0)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        store.project.drum_tracks.get_mut(&self.0)
    }

    fn as_enum(&self) -> Selector {
        Selector::DrumTrack(self.0)
    }
}