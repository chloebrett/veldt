use chrono::NaiveDateTime;
use crate::types::{Decibels, KnobPosition, Seconds, Volume, Beats, Freq};
use std::collections::BTreeSet;
use crate::model::wave_type::WaveType;
use crate::model::adsr_envelope::AdsrEnvelope;

type TrackId = usize;
type SampleId = usize;
type EffectId = usize;
type GeneratorInstanceId = usize;
type EffectInstanceId = usize;

struct AppConfig {
    sample_rate: u32,
}

struct Project {
    pub name: String,

    pub filename: String,

    pub created: NaiveDateTime,

    pub last_modified: NaiveDateTime,

    // TODO: info about user who owns and share permissions

    /// Ordered based on start_position.
    pub track_placements: BTreeSet<TrackPlacement>,

    pub samples: Vec<Sample>,

    // Maybe should be Vec<Box<dyn Generator>>?
    pub generators: Vec<GeneratorInstance>,

    pub mixer: Vec<MixerChannel>,
}

enum Generator {
    SingleWave {
        wave_generator: WaveGenerator,
    },

    MultiWave {
        wave_generators: Vec<WaveGenerator>,
    }
}

struct WaveGenerator {
    kind: WaveType,

    envelope: AdsrEnvelope,
}

struct GeneratorInstance {
    id: GeneratorInstanceId,

    generator: Generator,

    meta: GeneratorMeta,
}

struct GeneratorMeta {
    volume: Volume,

    // TODO: pan
}

struct TrackPlacement {
    track_id: TrackId,

    /// The time that the track starts within the arrangement.
    start_position: Beats,

    /// If None, then duration is not clipped.
    clipped_duration: Option<Beats>,

    /// Position that the track should be displayed visually, useful if there are overlapping tracks.
    /// Zero is the top.
    visual_placement: u32,
}

struct Sample {
    pub data: Vec<f32>,

    // TODO: instead store sample rate, and then derive this from the size of the data vec?
    pub duration_seconds: f32,

    // TODO: consider sample-specific sample rate.
    // TODO: consider multi-channel samples.
}

struct EffectInstance {
    effect: Effect,

    meta: EffectMeta,

    // TODO: automation links
}

enum Effect {
    // TODO: add basic effects with hard coded params.
    // Delay could be a good first one.
    // Speed shifting is another interesting one.
    // Phase inversion would be easy (just invert the amplitude) but hard to actually hear the
    // difference.
    // More complex: reverb, EQ, compressor, distortion, resampling.

    Delay {
        /// Volume of the first delayed repeat.
        volume: Volume,

        /// Proportional reduction of subsequent repeats.
        /// e.g. if volume = 0.8 and multiplier = 0.5, then the delay would be:
        /// 0.8 -> 0.4 -> 0.2 -> ...
        multiplier: KnobPosition,

        /// Limit to the number of delayed repeats.
        count: Option<u32>,
    },
    SimpleEq {
        kind: EqType,

        freq: Freq,

        q_value: KnobPosition,
    },
    SimpleCompressor {
        threshold: Decibels,

        attack: Seconds,

        release: Seconds,

        ratio: KnobPosition,

        gain: Decibels,
    }
}

enum EqType {
    Pass {
        kind: PassType,
    },
    Notch,
    Shelf{kind: ShelfType, amount: Decibels}
}

enum ShelfType {
    Low,
    High,
}

enum PassType {
    Low,
    High,
    Band
}

struct EffectMeta {
    id: EffectId,

    wet: KnobPosition,

    // TODO: pan
}

struct MixerChannel {
    effects: Vec<EffectInstance>
}
