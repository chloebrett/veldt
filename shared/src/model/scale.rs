use crate::pmodel::*;

use strum::{Display, EnumIter, EnumString};
use local_macro::{FromProto, IntoProto};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString, Display, EnumIter, FromProto, IntoProto
)]
pub enum Scale {
    Chromatic,
    Major,
    #[strum(serialize = "Natural Minor")]
    NaturalMinor,
    #[strum(serialize = "Harmonic Minor")]
    HarmonicMinor,
    Pentatonic,
}
