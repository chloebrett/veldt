use crate::model::EqConfig;
use crate::pmodel::{
    CompressorConfigProto, DelayConfigProto, EffectInstanceProto, EffectMetaProto,
    SimpleCompressorProto, SimpleDelayProto, SimpleEqProto, effect_instance_proto,
};
use crate::types::{Decibels, KnobPosition, Milliseconds, Volume};

type EffectId = usize;
type _EffectInstanceId = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct EffectInstance {
    pub effect: Effect,

    pub meta: EffectMeta,
    // TODO: automation links
}

impl From<EffectInstanceProto> for EffectInstance {
    fn from(item: EffectInstanceProto) -> Self {
        EffectInstance {
            meta: item.meta.unwrap().into(),
            effect: match item.effect.unwrap() {
                effect_instance_proto::Effect::SimpleDelay(simple_delay) => Effect::SimpleDelay {
                    config: simple_delay.config.unwrap().into(),
                },
                effect_instance_proto::Effect::SimpleEq(simple_eq) => Effect::SimpleEq {
                    config: simple_eq.config.unwrap().into(),
                },
                effect_instance_proto::Effect::SimpleCompressor(simple_compressor) => {
                    Effect::SimpleCompressor {
                        config: simple_compressor.config.unwrap().into(),
                    }
                }
            },
        }
    }
}

impl From<EffectInstance> for EffectInstanceProto {
    fn from(item: EffectInstance) -> Self {
        EffectInstanceProto {
            meta: Some(item.meta.into()),
            effect: match item.effect {
                Effect::SimpleDelay { config } => Some(effect_instance_proto::Effect::SimpleDelay(
                    SimpleDelayProto {
                        config: Some(config.into()),
                    },
                )),
                Effect::SimpleEq { config } => {
                    Some(effect_instance_proto::Effect::SimpleEq(SimpleEqProto {
                        config: Some(config.into()),
                    }))
                }
                Effect::SimpleCompressor { config } => Some(
                    effect_instance_proto::Effect::SimpleCompressor(SimpleCompressorProto {
                        config: Some(config.into()),
                    }),
                ),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    SimpleDelay { config: DelayConfig },
    // simple as opposed to parametric.
    SimpleEq { config: EqConfig },
    SimpleCompressor { config: CompressorConfig },
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectMeta {
    pub id: EffectId,

    pub wet: KnobPosition,
    // TODO: pan
}

impl From<EffectMetaProto> for EffectMeta {
    fn from(item: EffectMetaProto) -> Self {
        EffectMeta {
            id: item.id as usize,
            wet: item.wet,
        }
    }
}

impl From<EffectMeta> for EffectMetaProto {
    fn from(item: EffectMeta) -> Self {
        EffectMetaProto {
            id: item.id as u32,
            wet: item.wet,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DelayConfig {
    pub amplitude: Volume,

    pub delay_ms: Milliseconds,
}

impl From<DelayConfigProto> for DelayConfig {
    fn from(item: DelayConfigProto) -> Self {
        DelayConfig {
            amplitude: item.amplitude,
            delay_ms: item.delay_ms,
        }
    }
}

impl From<DelayConfig> for DelayConfigProto {
    fn from(item: DelayConfig) -> Self {
        DelayConfigProto {
            amplitude: item.amplitude,
            delay_ms: item.delay_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompressorConfig {
    threshold: Decibels,

    attack: Milliseconds,

    release: Milliseconds,

    ratio: KnobPosition,

    gain: Decibels,
}

impl From<CompressorConfigProto> for CompressorConfig {
    fn from(item: CompressorConfigProto) -> Self {
        CompressorConfig {
            threshold: item.threshold,
            attack: item.attack,
            release: item.release,
            ratio: item.ratio,
            gain: item.gain,
        }
    }
}

impl From<CompressorConfig> for CompressorConfigProto {
    fn from(item: CompressorConfig) -> Self {
        CompressorConfigProto {
            threshold: item.threshold,
            attack: item.attack,
            release: item.release,
            ratio: item.ratio,
            gain: item.gain,
        }
    }
}
