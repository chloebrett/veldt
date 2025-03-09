use crate::types::{Decibels, KnobPosition, Milliseconds, Volume};
use crate::model::EqConfig;

type EffectId = usize;
type _EffectInstanceId = usize;

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

pub struct EffectMeta {
    pub id: EffectId,

    pub wet: KnobPosition,
    // TODO: pan
}

pub struct DelayConfig {
    pub amplitude: Volume,

    pub delay_ms: Milliseconds,
}

pub struct _CompressorConfig {
    _threshold: Decibels,

    _attack: Milliseconds,

    _release: Milliseconds,

    _ratio: KnobPosition,

    _gain: Decibels,
}
