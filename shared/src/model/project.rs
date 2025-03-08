use crate::model::{AdsrEnvelope, EqType, WaveType};
use crate::types::{Beats, Decibels, Freq, KnobPosition, Milliseconds, Volume};
use chrono::NaiveDateTime;
use std::collections::BTreeSet;

type _TrackId = usize;
type _SampleId = usize;
type EffectId = usize;
type _GeneratorInstanceId = usize;
type _EffectInstanceId = usize;

struct _AppConfig {
    sample_rate: u32,
}

struct _Project {
    pub name: String,

    pub filename: String,

    pub created: NaiveDateTime,

    pub last_modified: NaiveDateTime,

    // TODO: info about user who owns and share permissions
    /// Ordered based on start_position.
    pub track_placements: BTreeSet<_TrackPlacement>,

    pub samples: Vec<_Sample>,

    // Maybe should be Vec<Box<dyn Generator>>?
    pub generators: Vec<_GeneratorInstance>,

    pub mixer: Vec<MixerChannel>,
}

enum _Generator {
    SingleWave {
        wave_generator: _WaveGenerator,
    },

    MultiWave {
        wave_generators: Vec<_WaveGenerator>,
    },
}

struct _WaveGenerator {
    kind: WaveType,

    envelope: AdsrEnvelope,
}

struct _GeneratorInstance {
    id: _GeneratorInstanceId,

    generator: _Generator,

    meta: _GeneratorMeta,
}

struct _GeneratorMeta {
    volume: Volume,
    // TODO: pan
}

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

struct _Sample {
    pub data: Vec<f32>,

    // TODO: instead store sample rate, and then derive this from the size of the data vec?
    pub duration_seconds: f32,
    // TODO: consider sample-specific sample rate.
    // TODO: consider multi-channel samples.
}

pub struct EffectInstance {
    pub effect: Effect,

    pub meta: EffectMeta,
    // TODO: automation links
}

pub enum Effect {
    SimpleDelay { config: DelayConfig },
    // simple as opposed to parametric.
    SimpleEq { config: EqConfig },
    SimpleCompressor { config: _CompressorConfig },
}

pub struct DelayConfig {
    pub amplitude: Volume,

    pub delay_ms: Milliseconds,
}

pub struct EqConfig {
    pub kind: EqType,

    pub fc: Freq,

    pub q: KnobPosition,
}

pub struct _CompressorConfig {
    _threshold: Decibels,

    _attack: Milliseconds,

    _release: Milliseconds,

    _ratio: KnobPosition,

    _gain: Decibels,
}

pub struct EffectMeta {
    pub id: EffectId,

    pub wet: KnobPosition,
    // TODO: pan
}

pub struct MixerChannel {
    pub effects: Vec<EffectInstance>,
}
