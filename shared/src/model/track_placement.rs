use crate::model::TrackId;
use crate::pmodel::*;
use crate::types::Beats;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackPlacement {
    pub track_id: TrackId,

    /// The time that the track starts within the arrangement.
    pub offset: OrderedFloat<Beats>,

    /// If None, then duration is not clipped.
    pub clipped_duration: Option<OrderedFloat<Beats>>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    pub visual_placement: u32,
}

impl PartialOrd for TrackPlacement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TrackPlacement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl From<TrackPlacementProto> for TrackPlacement {
    fn from(item: TrackPlacementProto) -> Self {
        TrackPlacement {
            track_id: item.track_id,
            offset: item.offset.into(),
            clipped_duration: item.clipped_duration.map(OrderedFloat),
            visual_placement: item.visual_placement,
        }
    }
}

impl From<TrackPlacement> for TrackPlacementProto {
    fn from(item: TrackPlacement) -> Self {
        TrackPlacementProto {
            track_id: item.track_id,
            offset: *item.offset,
            clipped_duration: item.clipped_duration.map(|it| *it),
            visual_placement: item.visual_placement,
        }
    }
}
