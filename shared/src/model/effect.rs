use crate::model::{EqConfig, WaveType};
use crate::pmodel::{
    CompressorConfigProto, DelayConfigProto, EffectInstanceProto, EffectMetaProto,
    ModDelayConfigProto, ModDelayProto, SimpleCompressorProto, SimpleDelayProto, SimpleEqProto,
    effect_instance_proto,
};
use crate::types::{Freq, KnobPosition, Milliseconds, Volume};
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
            EffectProto::ModDelay(mod_delay) => Effect::ModDelay {
                config: mod_delay.config.unwrap().into(),
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
            Effect::ModDelay { config } => EffectProto::ModDelay(ModDelayProto {
                config: Some(config.into()),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    SimpleDelay { config: DelayConfig },
    // simple as opposed to parametric.
    SimpleEq { config: EqConfig },
    SimpleCompressor { config: CompressorConfig },
    // Modulated delay, e.g. vibrato, flanger, chorus.
    ModDelay { config: ModDelayConfig },
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EffectMeta {
    #[proto_type_u32]
    pub id: EffectId,
    pub wet: KnobPosition,
    pub mute: bool,
    // TODO: pan
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct DelayConfig {
    pub delay_ms: Milliseconds,
}

/// Modulated delay, e.g. vibrato, flanger, chorus.
#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct ModDelayConfig {
    pub min_depth: u32, // Samples
    pub max_depth: u32, // Samples
    pub freq: Freq,     // LFO rate
    #[proto_enum]
    pub lfo_type: WaveType,
    // No support for feedback for now.
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct CompressorConfig {
    // TODO: use Decibels instead of Volume.
    pub threshold: Volume,
    pub attack_ms: Milliseconds,
    pub release_ms: Milliseconds,
    pub ratio: KnobPosition,
    pub gain: Volume,
}

#[cfg(test)]
// Test effect configs as they may not appear in the Project test.
mod tests {
    use super::*;
    use crate::testing::proto::proto_testing::assert_proto_round_trip;

    #[test]
    fn delay_config_proto_round_trip() {
        let delay_config = DelayConfig { delay_ms: 0.2 };
        assert_proto_round_trip::<DelayConfig, DelayConfigProto>(delay_config);
    }

    #[test]
    fn compressor_config_proto_round_trip() {
        let compressor_config = CompressorConfig {
            threshold: 0.3,
            attack_ms: 0.2,
            release_ms: 0.6,
            ratio: 0.2,
            gain: 1.0,
        };
        assert_proto_round_trip::<CompressorConfig, CompressorConfigProto>(compressor_config);
    }
}
