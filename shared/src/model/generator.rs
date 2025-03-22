use crate::model::{AdsrEnvelope, WaveType};
use crate::pmodel::SimpleWaveProto;
use crate::pmodel::{
    GeneratorInstanceProto, GeneratorMetaProto, SimpleWaveConfigProto,
    generator_instance_proto::Kind,
};
use crate::types::Volume;

type GeneratorInstanceId = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorInstance {
    pub id: GeneratorInstanceId,

    pub kind: GeneratorType,

    pub meta: GeneratorMeta,
}

impl From<GeneratorInstanceProto> for GeneratorInstance {
    fn from(item: GeneratorInstanceProto) -> Self {
        GeneratorInstance {
            id: item.id as usize,
            meta: item.meta.unwrap().into(),
            kind: match item.kind.unwrap() {
                Kind::SimpleWave(config) => GeneratorType::SimpleWave {
                    config: config.config.unwrap().into(),
                },
            },
        }
    }
}

impl From<GeneratorInstance> for GeneratorInstanceProto {
    fn from(item: GeneratorInstance) -> Self {
        GeneratorInstanceProto {
            id: item.id as u32,
            meta: Some(item.meta.into()),
            kind: match item.kind {
                GeneratorType::SimpleWave { config } => Some(Kind::SimpleWave(SimpleWaveProto {
                    config: Some(config.into()),
                })),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorType {
    SimpleWave { config: SimpleWaveConfig },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleWaveConfig {
    pub wave: WaveType,

    pub envelope: AdsrEnvelope,

    pub osc_count: u32,

    pub detune_cents: f32,
}

impl From<SimpleWaveConfigProto> for SimpleWaveConfig {
    fn from(item: SimpleWaveConfigProto) -> Self {
        SimpleWaveConfig {
            wave: item.wave().into(),
            envelope: item.envelope.unwrap().into(),
            osc_count: item.osc_count,
            detune_cents: item.detune_cents,
        }
    }
}

impl From<SimpleWaveConfig> for SimpleWaveConfigProto {
    fn from(item: SimpleWaveConfig) -> Self {
        SimpleWaveConfigProto {
            wave: item.wave as i32,
            envelope: Some(item.envelope.into()),
            osc_count: item.osc_count,
            detune_cents: item.detune_cents,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorMeta {
    pub volume: Volume,
    // TODO: pan
}

impl From<GeneratorMetaProto> for GeneratorMeta {
    fn from(item: GeneratorMetaProto) -> Self {
        GeneratorMeta {
            volume: item.volume,
        }
    }
}

impl From<GeneratorMeta> for GeneratorMetaProto {
    fn from(item: GeneratorMeta) -> Self {
        GeneratorMetaProto {
            volume: item.volume,
        }
    }
}
