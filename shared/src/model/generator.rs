use crate::model::{AdsrEnvelope, WaveType};
use crate::pmodel::{
    AntiAliasingModeProto, GeneratorInstanceProto, GeneratorMetaProto, NoiseConfigProto,
    NoiseProto, NoiseTypeProto, OscillatorConfigProto, SimpleWaveConfigProto, SimpleWaveProto,
    TripleOscConfigProto, TripleOscProto, generator_instance_proto::Kind as GeneratorTypeProto,
};
use crate::types::{KnobPosition, Volume};
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

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
            GeneratorType::Noise { config } => GeneratorTypeProto::Noise(NoiseProto {
                config: Some(config.into()),
            }),
            GeneratorType::TripleOsc { config } => GeneratorTypeProto::TripleOsc(TripleOscProto {
                config: Some(config.into()),
            }),
        }
    }
}

impl From<GeneratorTypeProto> for GeneratorType {
    fn from(item: GeneratorTypeProto) -> GeneratorType {
        match item {
            GeneratorTypeProto::SimpleWave(config) => GeneratorType::SimpleWave {
                config: config.config.unwrap().into(),
            },
            GeneratorTypeProto::Noise(config) => GeneratorType::Noise {
                config: config.config.unwrap().into(),
            },
            GeneratorTypeProto::TripleOsc(config) => GeneratorType::TripleOsc {
                config: config.config.unwrap().into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorType {
    SimpleWave { config: SimpleWaveConfig },
    Noise { config: NoiseConfig },
    TripleOsc { config: TripleOscConfig },
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct SimpleWaveConfig {
    #[proto_enum]
    pub wave: WaveType,

    #[proto_optional]
    pub envelope: AdsrEnvelope,

    pub osc_count: u32,

    pub detune_cents: f32,

    #[proto_enum]
    pub anti_aliasing_mode: AntiAliasingMode,

    pub oversample_factor: u32,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct NoiseConfig {
    #[proto_enum]
    pub kind: NoiseType,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub enum NoiseType {
    White,
    Brown,
    Pink,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TripleOscConfig {
    pub oscillators: [OscillatorConfig; 3],
}

impl From<TripleOscConfigProto> for TripleOscConfig {
    fn from(proto: TripleOscConfigProto) -> Self {
        let vec = proto.oscillators;
        assert_eq!(
            vec.len(),
            3,
            "TripleOscConfig must have exactly 3 oscillators"
        );
        TripleOscConfig {
            oscillators: [
                vec[0].clone().into(),
                vec[1].clone().into(),
                vec[2].clone().into(),
            ],
        }
    }
}

impl From<TripleOscConfig> for TripleOscConfigProto {
    fn from(config: TripleOscConfig) -> Self {
        TripleOscConfigProto {
            oscillators: config
                .oscillators
                .iter()
                .map(|osc| osc.clone().into())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct OscillatorConfig {
    #[proto_enum]
    pub wave: WaveType,

    pub volume: Volume,

    pub pan: KnobPosition,

    pub coarse_detune: i32,

    pub fine_detune: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumString, Display, EnumIter, IntoProto, FromProto)]
pub enum AntiAliasingMode {
    // No anti-aliasing. Uses naive waves without oversampling. Produces artifacts for waves like
    // square and saw, especially at high frequencies.
    Off,

    // Reduces aliasing by oversampling by the given factor, low passing, then downsampling. This uses additional computation
    // power.
    Oversample,

    // Avoids aliasing entirely by constructing the waveform additively. This results in zero
    // aliasing but is the most computationally expensive.
    Additive,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct GeneratorMeta {
    pub volume: Volume,

    pub mute: bool,

    pub pan: KnobPosition,
}
