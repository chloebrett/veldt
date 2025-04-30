use crate::model::{AdsrEnvelope, LfoConfig, WaveType};
use crate::pmodel::{
    AntiAliasingModeProto, GeneratorInstanceProto, GeneratorMetaProto, NoiseConfigProto,
    NoiseProto, NoiseTypeProto, OscillatorConfigProto, SimpleWaveConfigProto, SimpleWaveProto,
    SubSynthConfigProto, SubSynthProto, generator_instance_proto::Kind as GeneratorTypeProto,
};
use crate::serialize::map_vec;
use crate::types::{KnobPosition, Volume};
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

use super::ModMatrix;

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
    fn from(item: GeneratorType) -> Self {
        match item {
            GeneratorType::SimpleWave(config) => Self::SimpleWave(SimpleWaveProto {
                config: Some(config.into()),
            }),
            GeneratorType::Noise(config) => Self::Noise(NoiseProto {
                config: Some(config.into()),
            }),
            GeneratorType::SubSynth(config) => Self::SubSynth(SubSynthProto {
                config: Some(config.into()),
            }),
        }
    }
}

impl From<GeneratorTypeProto> for GeneratorType {
    fn from(item: GeneratorTypeProto) -> Self {
        match item {
            GeneratorTypeProto::SimpleWave(config) => {
                Self::SimpleWave(config.config.unwrap().into())
            }
            GeneratorTypeProto::Noise(config) => Self::Noise(config.config.unwrap().into()),
            GeneratorTypeProto::SubSynth(config) => Self::SubSynth(config.config.unwrap().into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorType {
    SimpleWave(SimpleWaveConfig),
    Noise(NoiseConfig),
    SubSynth(SubSynthConfig),
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

#[derive(Clone, Debug, PartialEq, IntoProto)]
pub struct SubSynthConfig {
    #[proto_repeated]
    pub oscillators: [OscillatorConfig; 3],

    #[proto_repeated]
    pub envelopes: [AdsrEnvelope; 3],

    #[proto_repeated]
    pub lfos: [LfoConfig; 3],

    #[proto_optional]
    pub matrix_config: ModMatrix,
}

impl From<SubSynthConfigProto> for SubSynthConfig {
    fn from(proto: SubSynthConfigProto) -> Self {
        let SubSynthConfigProto {
            oscillators,
            envelopes,
            lfos,
            matrix_config,
        } = proto;

        SubSynthConfig {
            oscillators: map_vec(oscillators)
                .try_into()
                .expect("Expected 3 oscillators!"),
            envelopes: map_vec(envelopes)
                .try_into()
                .expect("Expected 3 envelopes!"),
            lfos: map_vec(lfos).try_into().expect("Expected 3 LFOs!"),
            matrix_config: matrix_config.unwrap().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct OscillatorConfig {
    #[proto_enum]
    pub wave: WaveType,

    pub volume: Volume,

    pub pan: KnobPosition,

    pub osc_detune: f32,

    pub osc_count: u32,

    pub unison_detune: f32,
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
