use crate::pmodel::{DrumTrackProto, PlacedDrumProto};
use crate::model::SampleId;
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use crate::serialize::map_vec;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Default)]
pub struct Drum {
    /// Ordered by offset.
    pub sample_id: SampleId,
    #[proto_repeated]
    pub drums: Vec<PlacedDrum>,
    pub offset: OrderedFloat<Beats>,
}

/// Ordered by offset.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromProto, IntoProto)]
pub struct PlacedDrum {
    //represents a single drum hit
    pub offset: OrderedFloat<Beats>,
}


impl Drum {
    pub fn unclipped_duration(&self) -> OrderedFloat<Beats> {
        self.drums
            .iter()
            .map(|drum| drum.offset)
            .max_by(|x, y| x.cmp(y))
            .unwrap_or(OrderedFloat(0.0))
    }
}

impl From<DrumTrackProto> for Drum {
    fn from(item: DrumTrackProto) -> Self {
        Self {
            sample_id: item.sample_id.into(),
            drums: item.drums.into_iter().map(|d| d.into()).collect(),
            offset: OrderedFloat(item.offset),
        }
    }
}

impl From<Drum> for DrumTrackProto {
    fn from(item: Drum) -> Self {
        Self {
            sample_id: item.sample_id.into(),
            drums: item.drums.into_iter().map(|d| d.into()).collect(),
            offset: *item.offset,
        }
    }
}

