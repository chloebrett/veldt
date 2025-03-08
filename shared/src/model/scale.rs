use crate::pmodel::*;

use strum::{Display, EnumIter, EnumString};

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
            ScaleProto::ChromaticScale => Scale::Chromatic,
            ScaleProto::MajorScale => Scale::Major,
            ScaleProto::HarmonicMinorScale => Scale::HarmonicMinor,
            ScaleProto::NaturalMinorScale => Scale::NaturalMinor,
            ScaleProto::PentatonicScale => Scale::Pentatonic,
            ScaleProto::UnknownScale => panic!(),
        }
    }
}

impl From<Scale> for ScaleProto {
    fn from(item: Scale) -> Self {
        match item {
            Scale::Chromatic => ScaleProto::ChromaticScale,
            Scale::Major => ScaleProto::MajorScale,
            Scale::HarmonicMinor => ScaleProto::HarmonicMinorScale,
            Scale::NaturalMinor => ScaleProto::NaturalMinorScale,
            Scale::Pentatonic => ScaleProto::PentatonicScale,
        }
    }
}
