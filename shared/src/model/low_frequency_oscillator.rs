use local_macro::{FromProto, IntoProto};
use crate::model::WaveType;
use crate::pmodel::LowFrequencyOscillatorProto;

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct LowFrequencyOscillator {
    #[proto_enum]
    pub wave: WaveType,
    pub frequency: f32,
}