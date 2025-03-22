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

impl From<ScaleValue> for PitchValue {
    fn from(val: ScaleValue) -> Self {
        match val {
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
