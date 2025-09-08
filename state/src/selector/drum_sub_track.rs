use super::{Selector, SelectorTrait};
use crate::StoreData;
use crate::selector::{DrumTrackId, SampleId};
use shared::model::DrumSubTrack;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct DrumSubTrackSelector(pub DrumTrackId, pub SampleId);

impl SelectorTrait for DrumSubTrackSelector {
    type Item = DrumSubTrack;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let drum_track = store.project.drum_tracks.get(&self.0).unwrap();
        drum_track.drum_sub_tracks.get(&self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let drum_track = store.project.drum_tracks.get_mut(&self.0).unwrap();
        drum_track.drum_sub_tracks.get_mut(&self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::DrumSubTrack(self.0, self.1)
    }
}