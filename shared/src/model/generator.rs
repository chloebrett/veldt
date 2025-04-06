use crate::model::{AdsrEnvelope, WaveType};
use crate::pmodel::{
    GeneratorInstanceProto, GeneratorMetaProto, OversampleProto, SimpleWaveConfigProto,
    SimpleWaveProto, generator_instance_proto::Kind as GeneratorTypeProto,
    simple_wave_config_proto::AntiAliasingMode as AntiAliasingModeProto,
};
use crate::putil::EmptyProto;
use crate::types::Volume;
use local_macro::{FromProto, IntoProto};

type GeneratorInstanceId = usize;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct GeneratorInstance {
    #[proto_type_u32]
    pub id: GeneratorInstanceId,

    #[proto_optional]
    pub kind: GeneratorType,

    #[proto_optional]
    pub meta: GeneratorMeta,
}

impl From<GeneratorType> for GeneratorTypeProto {
    fn from(item: GeneratorType) -> GeneratorTypeProto {
        match item {
            GeneratorType::SimpleWave { config } => {
                GeneratorTypeProto::SimpleWave(SimpleWaveProto {
                    config: Some(config.into()),
                })
            }
        }
    }
}

impl From<GeneratorTypeProto> for GeneratorType {
    fn from(item: GeneratorTypeProto) -> GeneratorType {
        match item {
            GeneratorTypeProto::SimpleWave(config) => GeneratorType::SimpleWave {
                config: config.config.unwrap().into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorType {
    SimpleWave { config: SimpleWaveConfig },
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct SimpleWaveConfig {
    #[proto_enum]
    pub wave: WaveType,

    #[proto_optional]
    pub envelope: AdsrEnvelope,

    pub osc_count: u32,

    pub detune_cents: f32,

    #[proto_optional]
    pub anti_aliasing_mode: AntiAliasingMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AntiAliasingMode {
    // No anti-aliasing. Uses naive waves without oversampling. Produces artifacts for waves like
    // square and saw, especially at high frequencies.
    Off,

    // Reduces aliasing by oversampling by the given factor, low passing, then downsampling. This uses additional computation
    // power.
    Oversample { factor: u32 },

    // Avoids aliasing entirely by constructing the waveform additively. This results in zero
    // aliasing but is the most computationally expensive.
    Additive,
}

impl From<AntiAliasingModeProto> for AntiAliasingMode {
    fn from(item: AntiAliasingModeProto) -> AntiAliasingMode {
        match item {
            AntiAliasingModeProto::Off(_) => AntiAliasingMode::Off,
            AntiAliasingModeProto::Oversample(OversampleProto { factor }) => {
                AntiAliasingMode::Oversample { factor }
            }
            AntiAliasingModeProto::Additive(_) => AntiAliasingMode::Additive,
        }
    }
}

impl From<AntiAliasingMode> for AntiAliasingModeProto {
    fn from(item: AntiAliasingMode) -> AntiAliasingModeProto {
        match item {
            AntiAliasingMode::Off => AntiAliasingModeProto::Off(EmptyProto {}),
            AntiAliasingMode::Oversample { factor } => {
                AntiAliasingModeProto::Oversample(OversampleProto { factor })
            }
            AntiAliasingMode::Additive => AntiAliasingModeProto::Additive(EmptyProto {}),
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct GeneratorMeta {
    pub volume: Volume,
    // TODO: pan
}
