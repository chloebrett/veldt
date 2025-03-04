use crate::{pmodel::*, types::PitchValue};
use std::str;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString, Display)]
pub enum ScaleValue {
    A,
    #[strum(serialize = "A#")]
    ASharp,
    B,
    C,
    #[strum(serialize = "C#")]
    CSharp,
    D,
    #[strum(serialize = "D#")]
    DSharp,
    E,
    F,
    #[strum(serialize = "F#")]
    FSharp,
    G,
    #[strum(serialize = "G#")]
    GSharp,
}

impl Into<PitchValue> for ScaleValue {
    fn into(self: Self) -> PitchValue {
        match self {
            ScaleValue::A => 0,
            ScaleValue::ASharp => 1,
            ScaleValue::B => 2,
            ScaleValue::C => 3,
            ScaleValue::CSharp => 4,
            ScaleValue::D => 5,
            ScaleValue::DSharp => 6,
            ScaleValue::E => 7,
            ScaleValue::F => 8,
            ScaleValue::FSharp => 9,
            ScaleValue::G => 10,
            ScaleValue::GSharp => 11,
        }
    }
}

impl From<PitchValue> for ScaleValue {
    fn from(pitch_value: PitchValue) -> Self {
        match pitch_value % 12 {
            0 => ScaleValue::A,
            1 => ScaleValue::ASharp,
            2 => ScaleValue::B,
            3 => ScaleValue::C,
            4 => ScaleValue::CSharp,
            5 => ScaleValue::D,
            6 => ScaleValue::DSharp,
            7 => ScaleValue::E,
            8 => ScaleValue::F,
            9 => ScaleValue::FSharp,
            10 => ScaleValue::G,
            11 => ScaleValue::GSharp,
            _ => panic!(""), // This should never happen.
        }
    }
}

impl From<ScaleValueProto> for ScaleValue {
    fn from(item: ScaleValueProto) -> Self {
        match item {
            ScaleValueProto::UnknownScaleValue => panic!(""),
            ScaleValueProto::AScaleValue => ScaleValue::A,
            ScaleValueProto::ASharpScaleValue => ScaleValue::ASharp,
            ScaleValueProto::BScaleValue => ScaleValue::B,
            ScaleValueProto::CScaleValue => ScaleValue::C,
            ScaleValueProto::CSharpScaleValue => ScaleValue::CSharp,
            ScaleValueProto::DScaleValue => ScaleValue::D,
            ScaleValueProto::DSharpScaleValue => ScaleValue::DSharp,
            ScaleValueProto::EScaleValue => ScaleValue::E,
            ScaleValueProto::FScaleValue => ScaleValue::F,
            ScaleValueProto::FSharpScaleValue => ScaleValue::FSharp,
            ScaleValueProto::GScaleValue => ScaleValue::G,
            ScaleValueProto::GSharpScaleValue => ScaleValue::GSharp,
        }
    }
}

impl From<ScaleValue> for ScaleValueProto {
    fn from(item: ScaleValue) -> Self {
        match item {
            ScaleValue::A => ScaleValueProto::AScaleValue,
            ScaleValue::ASharp => ScaleValueProto::ASharpScaleValue,
            ScaleValue::B => ScaleValueProto::BScaleValue,
            ScaleValue::C => ScaleValueProto::CScaleValue,
            ScaleValue::CSharp => ScaleValueProto::CSharpScaleValue,
            ScaleValue::D => ScaleValueProto::DScaleValue,
            ScaleValue::DSharp => ScaleValueProto::DSharpScaleValue,
            ScaleValue::E => ScaleValueProto::EScaleValue,
            ScaleValue::F => ScaleValueProto::FScaleValue,
            ScaleValue::FSharp => ScaleValueProto::FSharpScaleValue,
            ScaleValue::G => ScaleValueProto::GScaleValue,
            ScaleValue::GSharp => ScaleValueProto::GSharpScaleValue,
        }
    }
}
