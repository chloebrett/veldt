use crate::pmodel::*;

use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString, Display, EnumIter,
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

impl From<ScaleProto> for Scale {
    fn from(item: ScaleProto) -> Self {
        match item {
            ScaleProto::Chromatic => Scale::Chromatic,
            ScaleProto::Major => Scale::Major,
            ScaleProto::HarmonicMinor => Scale::HarmonicMinor,
            ScaleProto::NaturalMinor => Scale::NaturalMinor,
            ScaleProto::Pentatonic => Scale::Pentatonic,
            ScaleProto::UnknownScale => panic!(),
        }
    }
}

impl From<Scale> for ScaleProto {
    fn from(item: Scale) -> Self {
        match item {
            Scale::Chromatic => ScaleProto::Chromatic,
            Scale::Major => ScaleProto::Major,
            Scale::HarmonicMinor => ScaleProto::HarmonicMinor,
            Scale::NaturalMinor => ScaleProto::NaturalMinor,
            Scale::Pentatonic => ScaleProto::Pentatonic,
        }
    }
}
