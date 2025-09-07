use super::{PlacementSelector, Selector, SelectorTrait};
use crate::StoreData;
use crate::selector::{PlacementId, SampleId};
use shared::model::{DrumSubTrack, DrumTrack};

impl DrumSubTrackSelector {
    pub fn upcast(&self) -> PlacementSelector {
        PlacementSelector(self.0)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub struct DrumSubTrackSelector(pub PlacementId, pub SampleId);

impl SelectorTrait for DrumSubTrackSelector {
    type Item = DrumSubTrack;

    fn try_select<'a>(&'a self, store: &'a StoreData) -> Option<&'a Self::Item> {
        let placement = store.project.placements.get(&self.0)?;
        let drum_placement: &DrumTrack = (placement).try_into().ok()?;
        drum_placement.drum_sub_tracks.get(&self.1)
    }

    fn try_select_mut<'a>(&'a self, store: &'a mut StoreData) -> Option<&'a mut Self::Item> {
        let placement = store.project.placements.get_mut(&self.0)?;
        let drum_placement: &mut DrumTrack = (placement).try_into().ok()?;
        drum_placement.drum_sub_tracks.get_mut(&self.1)
    }

    fn as_enum(&self) -> Selector {
        Selector::DrumSubTrack(self.0, self.1)
    }
}