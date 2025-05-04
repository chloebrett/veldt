use crate::pmodel::{
    NoiseConfigProto, NoiseTypeProto,
};
use local_macro::{FromProto, IntoProto};


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
