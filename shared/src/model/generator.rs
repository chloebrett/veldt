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
            GeneratorType::SubSynth { config } => GeneratorTypeProto::SubSynth(SubSynthProto {
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
            GeneratorTypeProto::SubSynth(config) => GeneratorType::SubSynth {
                config: config.config.unwrap().into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorType {
    SimpleWave { config: SimpleWaveConfig },
    Noise { config: NoiseConfig },
    SubSynth { config: SubSynthConfig },
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
pub struct SubSynthConfig {
    pub oscillators: [OscillatorConfig; 3],
    pub envelopes: [AdsrEnvelope; 3],
    pub lfos: [LfoConfig; 3],
    pub matrix_config: ModMatrix,
}

impl From<SubSynthConfigProto> for SubSynthConfig {
    fn from(proto: SubSynthConfigProto) -> Self {
        let oscillator_vec = proto.oscillators;
        assert_eq!(
            oscillator_vec.len(),
            3,
            "SubSynthConfig must have exactly 3 oscillators"
        );

        let env_vec = proto.envelopes;
        assert_eq!(
            env_vec.len(),
            3,
            "SubSynthConfig must have exactly 3 envelopes"
        );

        let lfo_vec = proto.lfos;
        let mod_matrix = proto.matrix_config;
        assert_eq!(lfo_vec.len(), 3, "SubSynthConfig must have exactly 3 lfos");

        SubSynthConfig {
            oscillators: [
                oscillator_vec[0].into(),
                oscillator_vec[1].into(),
                oscillator_vec[2].into(),
            ],
            envelopes: [env_vec[0].into(), env_vec[1].into(), env_vec[2].into()],
            lfos: [lfo_vec[0].into(), lfo_vec[1].into(), lfo_vec[2].into()],
            matrix_config: mod_matrix.unwrap().into(),
        }
    }
}

impl From<SubSynthConfig> for SubSynthConfigProto {
    fn from(config: SubSynthConfig) -> Self {
        SubSynthConfigProto {
            oscillators: map_vec(config.oscillators.to_vec()),
            envelopes: map_vec(config.envelopes.to_vec()),
            lfos: map_vec(config.lfos.to_vec()),
            matrix_config: Some(config.matrix_config.into()),
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
