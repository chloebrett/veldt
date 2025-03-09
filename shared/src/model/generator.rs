use crate::model::{AdsrEnvelope, WaveType};
use crate::types::Volume;

type _GeneratorInstanceId = usize;

pub enum _Generator {
    SingleWave {
        wave_generator: _WaveGenerator,
    },

    MultiWave {
        wave_generators: Vec<_WaveGenerator>,
    },
}

pub struct _WaveGenerator {
    _kind: WaveType,

    _envelope: AdsrEnvelope,
}

pub struct _GeneratorInstance {
    _id: _GeneratorInstanceId,

    _generator: _Generator,

    _meta: _GeneratorMeta,
}

pub struct _GeneratorMeta {
    _volume: Volume,
    // TODO: pan
}
