use crate::model::adsr_envelope::AdsrEnvelope;
use crate::model::wave_type::WaveType;
use crate::types::{Beats, Decibels, Freq, KnobPosition, Milliseconds, Seconds, Volume};
use chrono::NaiveDateTime;
use std::collections::BTreeSet;

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
    SingleWave { wave_generator: WaveGenerator },

    MultiWave { wave_generators: Vec<WaveGenerator> },
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

pub struct EffectInstance {
    pub effect: Effect,

    pub meta: EffectMeta,
    // TODO: automation links
}

pub enum Effect {
    // TODO: add basic effects with hard coded params.
    // Delay could be a good first one.
    // Speed shifting is another interesting one.
    // Phase inversion would be easy (just invert the amplitude) but hard to actually hear the
    // difference.
    // More complex: reverb, EQ, compressor, distortion, phaser, resampling, and more...
    SimpleDelay {
        // Note: for now, only feed-forward (i.e. one repeat).
        // In future: support feedback delay, with multiple repeats.
        amplitude: Volume,

        delay_ms: Milliseconds,
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
    },
}

pub enum EqType {
    Pass { kind: PassType },
    Notch,
    Shelf { kind: ShelfType, amount: Decibels },
    BandStop,
}

enum ShelfType {
    Low,
    High,
}

pub enum PassType {
    Low {
        algorithm: LowHighPassAlgorithm,
    },
    High {
        algorithm: LowHighPassAlgorithm,
    },
    Band {
        algorithm: BandPassAlgorithm,
    },

    /// All-pass filter: flat frequency response, but has a phase response.
    /// Used as an intermediate component in some phasers and reverb.
    /// Designed using pole-zero pairs with reciprocal radii.
    All,
}

pub enum BandPassAlgorithm {
    /// A simple and efficient conjugate pole resonator. Suffers from asymmetry in its response.
    SimpleResonator,

    /// Smith-Angell resonator, which adds two zeros (at z=-1 and z=1) to limit asymmetry
    /// and make the band pass even more selective.
    SmithAngell,
}

enum LowHighPassAlgorithm {
    /// Standard filter with -3dB attenuation.
    ButterWorth,

    /// Linkwitz-Riley with -6dB attenuation.
    LinkwitzRiley,
}

pub struct EffectMeta {
    pub id: EffectId,

    pub wet: KnobPosition,
    // TODO: pan
}

struct MixerChannel {
    effects: Vec<EffectInstance>,
}
