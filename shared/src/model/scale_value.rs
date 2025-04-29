use crate::{pmodel::*, types::PitchValue};
use local_macro::{FromProto, IntoProto};
use std::str;
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
pub enum ScaleValue {
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
    A,
    #[strum(serialize = "A#")]
    ASharp,
    B,
}

impl From<ScaleValue> for PitchValue {
    fn from(val: ScaleValue) -> Self {
        match val {
            ScaleValue::C => 0,
            ScaleValue::CSharp => 1,
            ScaleValue::D => 2,
            ScaleValue::DSharp => 3,
            ScaleValue::E => 4,
            ScaleValue::F => 5,
            ScaleValue::FSharp => 6,
            ScaleValue::G => 7,
            ScaleValue::GSharp => 8,
            ScaleValue::A => 9,
            ScaleValue::ASharp => 10,
            ScaleValue::B => 11,
        }
    }
}

impl From<PitchValue> for ScaleValue {
    fn from(pitch_value: PitchValue) -> Self {
        match pitch_value % 12 {
            0 => Self::C,
            1 => Self::CSharp,
            2 => Self::D,
            3 => Self::DSharp,
            4 => Self::E,
            5 => Self::F,
            6 => Self::FSharp,
            7 => Self::G,
            8 => Self::GSharp,
            9 => Self::A,
            10 => Self::ASharp,
            11 => Self::B,
            _ => panic!("{}", pitch_value), // This should never happen.
        }
    }
}
