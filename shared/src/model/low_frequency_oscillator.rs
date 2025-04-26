use crate::model::WaveType;
use crate::pmodel::LfoConfigProto;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto, Copy)]
pub struct LfoConfig {
    #[proto_enum]
    pub wave: WaveType,
    pub frequency: f32,
}
