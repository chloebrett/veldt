use crate::pmodel::{
    CompressorProto, DelayProto, EffectInstanceProto, EffectMetaProto, ModDelayProto,
    SimpleEqProto, effect_instance_proto::It as EffectProto,
};
use crate::types::KnobPosition;
use local_macro::{FromProto, IntoProto};
use strum::{EnumIter, EnumString};

mod compressor;
mod delay;
mod eq;
mod mod_delay;

pub use compressor::*;
pub use delay::*;
pub use eq::*;
pub use mod_delay::*;

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
