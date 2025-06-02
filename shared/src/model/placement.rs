use crate::model::GeneratorId;
use crate::pmodel::{placement_proto::Kind as PlacementTypeProto, *};
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Placement {
    pub kind: PlacementType,

    /// The time that the track starts within the arrangement.
    pub offset: OrderedFloat<Beats>,

    /// If None, then duration is not clipped.
    pub clipped_duration: Option<OrderedFloat<Beats>>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    pub visual_placement: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementType {
    Track(TrackPlacement),
    Sample(SamplePlacement),
}

impl Default for PlacementType {
    fn default() -> Self {
        PlacementType::Track(TrackPlacement::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromProto, IntoProto, Default)]
pub struct TrackPlacement {
    /// TODO: use a more stable ID instead of the index.
    #[proto_type_u32]
    pub track_index: usize,

    /// The generator this track placement is associated with.
    #[proto_into]
    pub generator_id: GeneratorId,
}

impl<'a> TryFrom<&'a Placement> for &'a TrackPlacement {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::Track(it) => Ok(it),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FromProto, IntoProto, Default)]
pub struct SamplePlacement {
    /// TODO: use a more stable ID instead of the index.
    #[proto_type_u32]
    pub sample_index: usize,
}

impl<'a> TryFrom<&'a Placement> for &'a SamplePlacement {
    type Error = ();

    fn try_from(item: &'a Placement) -> Result<Self, ()> {
        match &item.kind {
            PlacementType::Sample(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl PartialOrd for Placement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Placement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl From<PlacementProto> for Placement {
    fn from(item: PlacementProto) -> Self {
        Self {
            kind: match item.kind.unwrap() {
                PlacementTypeProto::Track(it) => PlacementType::Track(it.into()),
                PlacementTypeProto::Sample(it) => PlacementType::Sample(it.into()),
            },
            offset: item.offset.into(),
            clipped_duration: item.clipped_duration.map(OrderedFloat),
            visual_placement: item.visual_placement,
        }
    }
}

impl From<Placement> for PlacementProto {
    fn from(item: Placement) -> Self {
        Self {
            kind: Some(match item.kind {
                PlacementType::Track(it) => PlacementTypeProto::Track(it.into()),
                PlacementType::Sample(it) => PlacementTypeProto::Sample(it.into()),
            }),
            offset: *item.offset,
            clipped_duration: item.clipped_duration.map(|it| *it),
            visual_placement: item.visual_placement,
        }
    }
}
