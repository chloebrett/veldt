use crate::pmodel::*;
use strum::{Display, EnumIter, EnumString};
use local_macro::{FromProto, IntoProto};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString, Display, EnumIter, FromProto, IntoProto
)]
pub enum WaveType {
    Sine,
    Square,
    Saw,
    Triangle, // TODO: also add a generator for white noise - but it's not constrained by freq.
}
