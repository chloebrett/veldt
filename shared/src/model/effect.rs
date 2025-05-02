use crate::model::{EqConfig, WaveType};
use crate::pmodel::{
    CompressorConfigProto, CompressorProto, DelayConfigProto, DelayProto, EffectInstanceProto,
    EffectMetaProto, ModDelayConfigProto, ModDelayProto, SimpleEqProto,
    effect_instance_proto::It as EffectProto,
};
use crate::types::{Freq, KnobPosition, Milliseconds, Volume};
use local_macro::{FromProto, IntoProto};
use strum::{EnumIter, EnumString};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EffectInstance {
    #[proto_optional]
    pub it: Effect,

    #[proto_optional]
    pub meta: EffectMeta,
    // TODO: automation links
}

impl From<EffectProto> for Effect {
    fn from(item: EffectProto) -> Self {
        match item {
            EffectProto::Delay(it) => Self::Delay(it.config.unwrap().into()),
            EffectProto::SimpleEq(it) => Self::SimpleEq(it.config.unwrap().into()),
            EffectProto::Compressor(it) => Self::Compressor(it.config.unwrap().into()),
            EffectProto::ModDelay(it) => Self::ModDelay(it.config.unwrap().into()),
        }
    }
}

impl From<Effect> for EffectProto {
    fn from(item: Effect) -> Self {
        match item {
            Effect::Delay(config) => Self::Delay(DelayProto {
                config: Some(config.into()),
            }),
            Effect::SimpleEq(config) => Self::SimpleEq(SimpleEqProto {
                config: Some(config.into()),
            }),
            Effect::Compressor(config) => Self::Compressor(CompressorProto {
                config: Some(config.into()),
            }),
            Effect::ModDelay(config) => Self::ModDelay(ModDelayProto {
                config: Some(config.into()),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, EnumString)]
pub enum Effect {
    Delay(DelayConfig),
    // Simple as opposed to parametric.
    SimpleEq(EqConfig),
    Compressor(CompressorConfig),
    // Modulated delay, e.g. vibrato, flanger, chorus.
    ModDelay(ModDelayConfig),
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct EffectMeta {
    pub wet: KnobPosition,
    pub mute: bool,
}

impl Default for EffectMeta {
    fn default() -> Self {
        Self {
            wet: 1.0,
            mute: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct DelayConfig {
    pub delay_ms: Milliseconds,
    pub feedback: Volume,
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            delay_ms: 100.0,
            feedback: 0.5,
        }
    }
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

impl Default for ModDelayConfig {
    fn default() -> Self {
        Self {
            min_depth: 100,
            max_depth: 300,
            freq: 10.0,
            lfo_type: WaveType::Triangle,
        }
    }
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

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            attack_ms: 30.0,
            release_ms: 30.0,
            ratio: 1.5,
            gain: 1.0,
        }
    }
}

#[cfg(test)]
// Test effect configs as they may not appear in the Project test.
mod tests {
    use super::*;
    use crate::testing::proto::proto_testing::assert_proto_round_trip;

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
