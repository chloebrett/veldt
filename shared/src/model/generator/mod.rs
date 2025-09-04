use crate::pmodel::{
    GeneratorInstanceProto, GeneratorMetaProto, NoiseProto, SimpleWaveProto, StingrayProto,
    generator_instance_proto::It as GeneratorProto,
};
use crate::types::{KnobPosition, Volume};
use local_macro::{FromProto, IntoProto};

mod alias;
mod noise;
mod oscillator;
mod polyphony;
mod simple_wave;
mod stingray;

pub use alias::*;
pub use noise::*;
pub use oscillator::*;
pub use polyphony::*;
pub use simple_wave::*;
pub use stingray::*;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct GeneratorInstance {
    #[proto_optional]
    pub it: Generator,

    #[proto_optional]
    pub meta: GeneratorMeta,
}

impl From<Generator> for GeneratorProto {
    fn from(item: Generator) -> Self {
        match item {
            Generator::SimpleWave(config) => Self::SimpleWave(SimpleWaveProto {
                config: Some(config.into()),
            }),
            Generator::Noise(config) => Self::Noise(NoiseProto {
                config: Some(config.into()),
            }),
            Generator::Stingray(config) => Self::Stingray(StingrayProto {
                config: Some(config.into()),
            }),
        }
    }
}

impl From<GeneratorProto> for Generator {
    fn from(item: GeneratorProto) -> Self {
        match item {
            GeneratorProto::SimpleWave(config) => Self::SimpleWave(config.config.unwrap().into()),
            GeneratorProto::Noise(config) => Self::Noise(config.config.unwrap().into()),
            GeneratorProto::Stingray(config) => Self::Stingray(config.config.unwrap().into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Generator {
    SimpleWave(SimpleWaveConfig),
    Noise(NoiseConfig),
    Stingray(StingrayConfig),
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct GeneratorMeta {
    pub volume: Volume,

    pub mute: bool,

    pub pan: KnobPosition,

    #[proto_type_u32]
    pub mixer_channel: usize,

    pub name: String,
}

impl Default for GeneratorMeta {
    fn default() -> Self {
        Self {
            volume: 1.0,
            mute: false,
            pan: 0.0,
            mixer_channel: 0,
            name: "".to_string(),
        }
    }
}
