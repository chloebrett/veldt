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
pub enum Scale {
    Chromatic,
    Major,
    #[strum(serialize = "Natural Minor")]
    NaturalMinor,
    #[strum(serialize = "Harmonic Minor")]
    HarmonicMinor,
    Pentatonic,
}

#[cfg(test)]
mod tests {
    use crate::tests::test_util::assert_proto_round_trip;

    use super::*;

    #[test]
    fn convert_scale_to_proto_and_back() {
        let scale = Scale::Pentatonic;
        assert_proto_round_trip::<Scale, ScaleProto>(scale);
    }
}
