use crate::model::{EffectInstance, GeneratorInstance, Track};
use crate::types::Beats;
use std::collections::BTreeSet;

type _TrackId = usize;
type _SampleId = usize;

struct _AppConfig {
    sample_rate: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,

    pub tracks: Vec<Track>,

    // TODO: info about user who owns and share permissions
    /// Ordered based on start_position.
    pub track_placements: BTreeSet<TrackPlacement>,

    pub samples: Vec<Sample>,

    pub generators: Vec<GeneratorInstance>,

    pub mixer: Vec<MixerChannel>,

    pub bpm: Beats,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackPlacement {
    track_id: _TrackId,

    /// The time that the track starts within the arrangement.
    start_position: Beats,

    /// If None, then duration is not clipped.
    clipped_duration: Option<Beats>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    visual_placement: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub data: Vec<f32>,

    pub sample_rate: f32,

    pub bit_depth: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MixerChannel {
    pub effects: Vec<EffectInstance>,
}
