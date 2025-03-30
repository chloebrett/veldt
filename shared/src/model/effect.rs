use crate::model::EqConfig;
use crate::pmodel::{
    CompressorConfigProto, DelayConfigProto, EffectInstanceProto, EffectMetaProto,
    SimpleCompressorProto, SimpleDelayProto, SimpleEqProto, effect_instance_proto,
};
use crate::types::{Decibels, KnobPosition, Milliseconds, Volume};
use effect_instance_proto::Effect as EffectProto;
use local_macro::{FromProto, IntoProto};

type EffectId = usize;
type _EffectInstanceId = usize;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EffectInstance {
    #[proto_optional]
    pub effect: Effect,

    #[proto_optional]
    pub meta: EffectMeta,
    // TODO: automation links
}

impl From<EffectProto> for Effect {
    fn from(item: EffectProto) -> Self {
        match item {
            EffectProto::SimpleDelay(simple_delay) => Effect::SimpleDelay {
                config: simple_delay.config.unwrap().into(),
            },
            EffectProto::SimpleEq(simple_eq) => Effect::SimpleEq {
                config: simple_eq.config.unwrap().into(),
            },
            EffectProto::SimpleCompressor(simple_compressor) => Effect::SimpleCompressor {
                config: simple_compressor.config.unwrap().into(),
            },
        }
    }
}

impl From<Effect> for EffectProto {
    fn from(item: Effect) -> Self {
        match item {
            Effect::SimpleDelay { config } => EffectProto::SimpleDelay(SimpleDelayProto {
                config: Some(config.into()),
            }),
            Effect::SimpleEq { config } => EffectProto::SimpleEq(SimpleEqProto {
                config: Some(config.into()),
            }),
            Effect::SimpleCompressor { config } => {
                EffectProto::SimpleCompressor(SimpleCompressorProto {
                    config: Some(config.into()),
                })
            }
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

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EffectMeta {
    #[proto_type_u32]
    pub id: EffectId,
    pub wet: KnobPosition,
    // TODO: pan
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct DelayConfig {
    pub amplitude: Volume,
    pub delay_ms: Milliseconds,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct CompressorConfig {
    threshold: Decibels,
    attack: Milliseconds,
    release: Milliseconds,
    ratio: KnobPosition,
    gain: Decibels,
}

#[cfg(test)]
mod tests {
    use crate::model::EqType;

    use super::*;

    #[test]
    fn convert_effect_instance_to_proto_and_back() {
        let effect_instance = EffectInstance {
            effect: Effect::SimpleEq {
                config: EqConfig {
                    kind: EqType::SimpleResonator,
                    fc: 1000.0,
                    q: 1.0,
                },
            },
            meta: EffectMeta { id: 0, wet: 1.0 },
        };
        let proto: EffectInstanceProto = effect_instance.clone().into();
        let result: EffectInstance = proto.into();
        assert_eq!(effect_instance, result);
    } 

    #[test]
    fn convert_effect_to_proto_and_back() {
        let effect = Effect::SimpleEq {
            config: EqConfig {
                kind: EqType::SimpleResonator,
                fc: 1000.0,
                q: 1.0,
            },
        };
        let proto: EffectProto = effect.clone().into();
        let result: Effect = proto.into();
        assert_eq!(effect, result);
    }

    #[test]
    fn convert_effect_meta_to_proto_and_back() {
        let effect_meta = EffectMeta { id: 0, wet: 1.0 };
        let proto: EffectMetaProto = effect_meta.clone().into();
        let result: EffectMeta = proto.into();
        assert_eq!(effect_meta, result);
    }

    #[test]
    fn convert_delay_config_to_proto_and_back() {
        let delay_config = DelayConfig {
            amplitude: 1.2,
            delay_ms: 0.2
        };
        let proto: DelayConfigProto = delay_config.clone().into();
        let result: DelayConfig = proto.into();
        assert_eq!(delay_config, result);
    }

    #[test]
    fn convert_compressor_config_to_proto_and_back() {
        let compressor_config = CompressorConfig {
            threshold: 0.3,attack: 0.2,release: 0.6, ratio: 0.2,gain: 1.0
        };
        let proto: CompressorConfigProto = compressor_config.clone().into();
        let result: CompressorConfig = proto.into();
        assert_eq!(compressor_config, result);
    }
}
