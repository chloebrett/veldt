use crate::model::{EffectInstance, GeneratorInstance, Track};
use crate::types::Beats;
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;

type _TrackId = usize;
type _SampleId = usize;

struct _AppConfig {
    sample_rate: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct _Project {
    pub name: String,

    pub filename: String,

    pub created: DateTime<Utc>,

    pub last_modified: DateTime<Utc>,

    pub tracks: Vec<Track>,

    // TODO: info about user who owns and share permissions
    /// Ordered based on start_position.
    pub track_placements: BTreeSet<_TrackPlacement>,

    pub samples: Vec<_Sample>,

    pub generators: Vec<GeneratorInstance>,

    pub mixer: Vec<MixerChannel>,
}

#[derive(Clone, Debug, PartialEq)]
struct _TrackPlacement {
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
struct _Sample {
    pub data: Vec<f32>,

    // TODO: instead store sample rate, and then derive this from the size of the data vec?
    pub duration_seconds: f32,
    // TODO: consider sample-specific sample rate.
    // TODO: consider multi-channel samples.
}

#[derive(Clone, Debug, PartialEq)]
pub struct MixerChannel {
    pub effects: Vec<EffectInstance>,
}
