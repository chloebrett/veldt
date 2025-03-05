use crate::pmodel::*;

use strum::{Display, EnumString, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString, Display, EnumIter)]
pub enum Scale {
    Chromatic, 
    Major, 
    #[strum(serialize = "Harmonic Minor")]
    HarmonicMinor 
}

impl From<ScaleProto> for Scale {
    fn from(item: ScaleProto) -> Self {
        match item {
            ScaleProto::Chromatic => Scale::Chromatic,
            ScaleProto::Major => Scale::Major,
            ScaleProto::HarmonicMinor => Scale::HarmonicMinor,
            ScaleProto::UnknownScale => panic!()
        }
    }
}

impl From<Scale> for ScaleProto {
    fn from(item: Scale) -> Self {
        match item {
            Scale::Chromatic => ScaleProto::Chromatic,
            Scale::Major => ScaleProto::Major,
            Scale::HarmonicMinor => ScaleProto::HarmonicMinor
        }
    }
}
