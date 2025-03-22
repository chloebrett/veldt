use crate::model::scale_value::ScaleValue;
use crate::pmodel::*;
use crate::types::*;
use local_macro::FromProto;
use std::fmt;
use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, FromProto)]
pub struct PitchName {
    pub scale_value: ScaleValue,
    pub octave: Octave,
}

impl From<PitchName> for PitchValue {
    fn from(val: PitchName) -> Self {
        let scale_value_pitch = val.scale_value as PitchValue;
        let octave_pitch = val.octave * 12;
        scale_value_pitch + octave_pitch
    }
}

impl From<PitchValue> for PitchName {
    fn from(pitch_value: PitchValue) -> Self {
        Self {
            scale_value: pitch_value.into(),
            octave: pitch_value / 12,
        }
    }
}

impl fmt::Display for PitchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.scale_value, self.octave)
    }
}

impl Add<PitchValue> for PitchName {
    type Output = Self;
    fn add(self: PitchName, other: PitchValue) -> PitchName {
        let pitch_value: PitchValue = self.into();
        (pitch_value + other).into()
    }
}

impl From<PitchName> for PitchNameProto {
    fn from(item: PitchName) -> Self {
        let scale_value: ScaleValueProto = item.scale_value.into();
        PitchNameProto {
            scale_value: scale_value as i32,
            octave: item.octave,
        }
    }
}
