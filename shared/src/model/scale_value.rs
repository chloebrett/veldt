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
            0 => ScaleValue::C,
            1 => ScaleValue::CSharp,
            2 => ScaleValue::D,
            3 => ScaleValue::DSharp,
            4 => ScaleValue::E,
            5 => ScaleValue::F,
            6 => ScaleValue::FSharp,
            7 => ScaleValue::G,
            8 => ScaleValue::GSharp,
            9 => ScaleValue::A,
            10 => ScaleValue::ASharp,
            11 => ScaleValue::B,
            _ => panic!("{}", pitch_value), // This should never happen.
        }
    }
}
