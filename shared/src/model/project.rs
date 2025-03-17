use crate::bytes::{as_bytes, as_floats};
use crate::model::{EffectInstance, GeneratorInstance, Track};
use crate::pmodel::*;
use crate::serialize::map_vec;
use crate::types::Beats;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

type _TrackId = usize;
type _SampleId = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,

    pub tracks: Vec<Track>,

    /// Ordered based on start_position.
    pub track_placements: Vec<TrackPlacement>,

    pub samples: Vec<Sample>,

    pub generators: Vec<GeneratorInstance>,

    pub mixer: Vec<MixerChannel>,

    pub bpm: Beats,
}

impl From<ProjectProto> for Project {
    fn from(item: ProjectProto) -> Self {
        Project {
            name: item.name,
            tracks: map_vec(item.tracks),
            track_placements: map_vec(item.track_placements),
            samples: map_vec(item.samples),
            generators: map_vec(item.generators),
            mixer: map_vec(item.mixer),
            bpm: item.bpm,
        }
    }
}

impl From<Project> for ProjectProto {
    fn from(item: Project) -> Self {
        ProjectProto {
            name: item.name,
            tracks: map_vec(item.tracks),
            track_placements: map_vec(item.track_placements),
            samples: map_vec(item.samples),
            generators: map_vec(item.generators),
            mixer: map_vec(item.mixer),
            bpm: item.bpm,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackPlacement {
    track_id: _TrackId,

    /// The time that the track starts within the arrangement.
    start_position: OrderedFloat<Beats>,

    /// If None, then duration is not clipped.
    clipped_duration: Option<OrderedFloat<Beats>>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    visual_placement: u32,
}

impl PartialOrd for TrackPlacement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TrackPlacement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_position.cmp(&other.start_position)
    }
}

impl From<TrackPlacementProto> for TrackPlacement {
    fn from(item: TrackPlacementProto) -> Self {
        TrackPlacement {
            track_id: item.track_id as usize,
            start_position: OrderedFloat(item.start_position),
            clipped_duration: item.clipped_duration.map(|it| OrderedFloat(it)),
            visual_placement: item.visual_placement,
        }
    }
}

impl From<TrackPlacement> for TrackPlacementProto {
    fn from(item: TrackPlacement) -> Self {
        TrackPlacementProto {
            track_id: item.track_id as u32,
            start_position: *item.start_position,
            clipped_duration: item.clipped_duration.map(|it| *it),
            visual_placement: item.visual_placement,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub data: Vec<f32>,

    pub sample_rate: f32,

    pub bit_depth: u32,
}

impl From<SampleProto> for Sample {
    fn from(item: SampleProto) -> Self {
        Sample {
            data: as_floats(&item.data),
            sample_rate: item.sample_rate,
            bit_depth: item.bit_depth,
        }
    }
}

impl From<Sample> for SampleProto {
    fn from(item: Sample) -> Self {
        SampleProto {
            data: as_bytes(&item.data),
            sample_rate: item.sample_rate,
            bit_depth: item.bit_depth,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MixerChannel {
    pub effects: Vec<EffectInstance>,
}

impl From<MixerChannelProto> for MixerChannel {
    fn from(item: MixerChannelProto) -> Self {
        MixerChannel {
            effects: map_vec(item.effects),
        }
    }
}

impl From<MixerChannel> for MixerChannelProto {
    fn from(item: MixerChannel) -> Self {
        MixerChannelProto {
            effects: map_vec(item.effects),
        }
    }
}
