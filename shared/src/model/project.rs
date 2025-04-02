use crate::bytes::{as_bytes, as_floats};
use crate::model::{EffectInstance, GeneratorInstance, Track};
use crate::pmodel::*;
use crate::types::Beats;
use local_macro::{FromProto, IntoProto};
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

pub type TrackId = usize;
type _SampleId = usize;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Project {
    pub name: String,

    #[proto_repeated]
    pub tracks: Vec<Track>,

    /// Ordered based on start_position.
    #[proto_repeated]
    pub track_placements: Vec<TrackPlacement>,

    #[proto_repeated]
    pub samples: Vec<Sample>,

    #[proto_repeated]
    pub generators: Vec<GeneratorInstance>,

    #[proto_repeated]
    pub mixer: Vec<MixerChannel>,

    pub bpm: Beats,
}

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
            track_id: item.track_id as usize,
            offset: item.offset.into(),
            clipped_duration: item.clipped_duration.map(OrderedFloat),
            visual_placement: item.visual_placement,
        }
    }
}

impl From<TrackPlacement> for TrackPlacementProto {
    fn from(item: TrackPlacement) -> Self {
        TrackPlacementProto {
            track_id: item.track_id as u32,
            offset: *item.offset,
            clipped_duration: item.clipped_duration.map(|it| *it),
            visual_placement: item.visual_placement,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub data: Vec<f32>,
    pub sample_rate: f32,
}

impl From<SampleProto> for Sample {
    fn from(item: SampleProto) -> Self {
        Sample {
            data: as_floats(&item.data),
            sample_rate: item.sample_rate,
        }
    }
}

impl From<Sample> for SampleProto {
    fn from(item: Sample) -> Self {
        SampleProto {
            data: as_bytes(&item.data),
            sample_rate: item.sample_rate,
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerChannel {
    #[proto_repeated]
    pub effects: Vec<EffectInstance>,
}
