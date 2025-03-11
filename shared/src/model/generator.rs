use crate::model::{AdsrEnvelope, WaveType};
use crate::types::Volume;

type GeneratorInstanceId = usize;

pub struct GeneratorInstance {
    pub id: GeneratorInstanceId,

    pub kind: GeneratorType,

    pub meta: GeneratorMeta,
}

pub enum GeneratorType {
    SimpleWave { config: SimpleWaveConfig },
}

pub struct SimpleWaveConfig {
    pub wave: WaveType,

    pub envelope: AdsrEnvelope,

    pub osc_count: u32,

    pub detune_cents: f32,
}

pub struct GeneratorMeta {
    pub volume: Volume,
    // TODO: pan
}
