use crate::model::adsr_envelope::AdsrEnvelope;
use crate::model::wave_type::WaveType;
use crate::types::{Beats, Decibels, Freq, KnobPosition, Milliseconds, Seconds, Volume};
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

// Somewhat duplicates the EqType hierarchy, but flattens it for display.
pub enum EqName {
    SimpleResonator,
    SmithAngellResonator,
    SimpleFirstOrderLowPass,
    SimpleFirstOrderHighPass,
    SimpleSecondOrderLowPass,
    SimpleSecondOrderHighPass,
    SimpleSecondOrderResonator,
    SimpleSecondOrderBandStop,
    ButterworthLowPass,
    ButterworthHighPass,
    ButterworthResonator,
    ButterworthBandStop,
}

pub enum EqType {
    Pass { kind: PassType },
    _Notch,
    _Shelf { kind: _ShelfType, amount: Decibels },
    BandStop { algorithm: BandStopAlgorithm },
}

pub enum _ShelfType {
    _Low,
    _High,
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

    /// Band pass variant of the second-order low/high pass algorithm.
    SimpleSecondOrder,
}

pub enum BandStopAlgorithm {
    // TODO: consolidate all the "simple second order" algorithms, etc.
    // Repeating their names in the type system could probably be avoided.
    SimpleSecondOrder,
}

pub enum LowHighPassAlgorithm {
    /// Standard filter with -3dB attenuation. Q is fixed at 0.707 and changing it has no effect.
    SimpleFirstOrder,

    /// Standard filter with -3dB attenuation.
    SimpleSecondOrder,

    /// Linkwitz-Riley with -6dB attenuation.
    LinkwitzRiley,
}

pub struct EffectMeta {
    pub id: EffectId,

    pub wet: KnobPosition,
    // TODO: pan
}

pub struct MixerChannel {
    pub effects: Vec<EffectInstance>,
}
