use crate::model::{AdsrEnvelope, WaveType};
use crate::types::Volume;

type GeneratorInstanceId = usize;

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorInstance {
    pub id: GeneratorInstanceId,

    pub kind: GeneratorType,

    pub meta: GeneratorMeta,
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

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorMeta {
    pub volume: Volume,
    // TODO: pan
}
