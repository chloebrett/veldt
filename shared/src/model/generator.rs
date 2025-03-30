use crate::model::{AdsrEnvelope, WaveType};
use crate::pmodel::SimpleWaveProto;
use crate::pmodel::{
    GeneratorInstanceProto, GeneratorMetaProto, SimpleWaveConfigProto,
    generator_instance_proto::Kind as GeneratorTypeProto,
};
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
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct GeneratorMeta {
    pub volume: Volume,
    // TODO: pan
}

#[cfg(test)] 
mod tests {
    use super::*;
    
    #[test]
    fn convert_generator_instance_to_proto_and_back() {
        let generator_instance = GeneratorInstance {
            id: 0,
            kind: GeneratorType::SimpleWave {
                config: SimpleWaveConfig {
                    wave: WaveType::Sine,
                    envelope: AdsrEnvelope {
                        attack: 0.1,
                        decay: 0.1,
                        sustain: 0.8,
                        release: 0.1,
                    },
                    osc_count: 4,
                    detune_cents: 5.0,
                },
            },
            meta: GeneratorMeta { volume: 1.0 },
        };
        let proto: GeneratorInstanceProto = generator_instance.clone().into();
        let result: GeneratorInstance = proto.into();
        assert_eq!(generator_instance, result)
    }

    #[test]
    fn convert_generator_type_to_proto_and_back() {
        let generator_type = GeneratorType::SimpleWave {
            config: SimpleWaveConfig {
                wave: WaveType::Sine,
                envelope: AdsrEnvelope {
                    attack: 0.1,
                    decay: 0.1,
                    sustain: 0.8,
                    release: 0.1,
                },
                osc_count: 4,
                detune_cents: 5.0,
            },
        };
        let proto: GeneratorTypeProto = generator_type.clone().into();
        let result: GeneratorType = proto.into();
        assert_eq!(generator_type, result);
    }

    #[test]
    fn convert_simple_wave_config_to_proto_and_back() {
        let simple_wave_config: SimpleWaveConfig = SimpleWaveConfig {
            wave: WaveType::Sine,
            envelope: AdsrEnvelope {
                attack: 0.1,
                decay: 0.1,
                sustain: 0.8,
                release: 0.1,
            },
            osc_count: 4,
            detune_cents: 5.0,
        };
        let proto: SimpleWaveConfigProto = simple_wave_config.clone().into();
        let result: SimpleWaveConfig = proto.into();
        assert_eq!(simple_wave_config, result);
    }

    #[test]
    fn convert_generator_meta_to_proto_and_back() {
        let generator_meta = GeneratorMeta {
            volume: 1.0,
        };
        let proto: GeneratorMetaProto = generator_meta.clone().into();
        let result: GeneratorMeta = proto.into();
        assert_eq!(generator_meta, result);
    }
}
