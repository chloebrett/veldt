use crate::model::EqConfig;
use crate::types::{Decibels, KnobPosition, Milliseconds, Volume};

type EffectId = usize;
type _EffectInstanceId = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct EffectInstance {
    pub effect: Effect,

    pub meta: EffectMeta,
    // TODO: automation links
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    SimpleDelay { config: DelayConfig },
    // simple as opposed to parametric.
    SimpleEq { config: EqConfig },
    SimpleCompressor { config: _CompressorConfig },
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectMeta {
    pub id: EffectId,

    pub wet: KnobPosition,
    // TODO: pan
}

#[derive(Clone, Debug, PartialEq)]
pub struct DelayConfig {
    pub amplitude: Volume,

    pub delay_ms: Milliseconds,
}

#[derive(Clone, Debug, PartialEq)]
pub struct _CompressorConfig {
    _threshold: Decibels,

    _attack: Milliseconds,

    _release: Milliseconds,

    _ratio: KnobPosition,

    _gain: Decibels,
}
