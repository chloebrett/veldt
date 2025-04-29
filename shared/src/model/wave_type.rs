use crate::pmodel::*;
use local_macro::{FromProto, IntoProto};
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    EnumIter,
    FromProto,
    IntoProto,
)]
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle,
}
