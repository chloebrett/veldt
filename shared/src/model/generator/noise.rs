use crate::model::NoiseType::White;
use crate::pmodel::{NoiseConfigProto, NoiseTypeProto};
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct NoiseConfig {
    #[proto_enum]
    pub kind: NoiseType,
}

#[derive(Clone, Copy, Debug, Display, EnumIter, PartialEq, FromProto, IntoProto)]
pub enum NoiseType {
    White,
    Brown,
    Pink,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self { kind: White }
    }
}
