use std::fmt;
use crate::{pmodel::*, types::PitchValue};
use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
pub enum ScaleValue {
    A,
    ASharp,
    B,
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
}

impl Into<PitchValue> for ScaleValue {
    fn into(self: Self) -> PitchValue {
        match self {
            ScaleValue::A => 0 as PitchValue,
            ScaleValue::ASharp => 1 as PitchValue,
            ScaleValue::B => 2 as PitchValue,
            ScaleValue::C => 3 as PitchValue,
            ScaleValue::CSharp => 4 as PitchValue,
            ScaleValue::D => 5 as PitchValue,
            ScaleValue::DSharp => 6 as PitchValue,
            ScaleValue::E => 7 as PitchValue,
            ScaleValue::F => 8 as PitchValue,
            ScaleValue::FSharp => 9 as PitchValue,
            ScaleValue::G => 10 as PitchValue,
            ScaleValue::GSharp => 11 as PitchValue
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
            _ => panic!("") // This should never happen.
        }
    }
}

impl fmt::Display for ScaleValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaleValue::A => write!(f, "A"),
            ScaleValue::ASharp => write!(f, "A#"),
            ScaleValue::B => write!(f, "B"),
            ScaleValue::C => write!(f, "C"),
            ScaleValue::CSharp => write!(f, "C#"),
            ScaleValue::D => write!(f, "D"),
            ScaleValue::DSharp => write!(f, "D#"),
            ScaleValue::E => write!(f, "E"),
            ScaleValue::F => write!(f, "F"),
            ScaleValue::FSharp => write!(f, "F#"),
            ScaleValue::G => write!(f, "G"),
            ScaleValue::GSharp => write!(f, "G#")
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
            ScaleValueProto::GSharpScaleValue => ScaleValue::GSharp
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
            ScaleValue::GSharp => ScaleValueProto::GSharpScaleValue
        }
    }
}
