use local_macro::{FromProto, IntoProto};
use crate::model::WaveType;
use crate::pmodel::LowFrequencyOscillatorConfigProto;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct LowFrequencyOscillatorConfig {
    #[proto_enum]
    pub wave: WaveType,
    pub frequency: f32,
}