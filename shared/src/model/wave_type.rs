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
    Triangle, // TODO: also add a generator for white noise - but it's not constrained by freq.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_from_wave_type_to_proto_and_back() {
        let wave_type = WaveType::Sine;
        let proto: WaveTypeProto = wave_type.into();
        let result: WaveType = proto.into();
        assert_eq!(wave_type, result);
    }

    #[test]
    fn convert_from_proto_to_wave_type_and_back() {
        let proto = WaveTypeProto::SineWaveType;
        let wave_type: WaveType = proto.into();
        let result: WaveTypeProto = wave_type.into();
        assert_eq!(proto, result);
    }
}
