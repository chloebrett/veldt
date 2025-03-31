use crate::model::scale_value::ScaleValue;
use crate::pmodel::*;
use crate::types::*;
use local_macro::{FromProto, IntoProto};
use std::fmt;
use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, FromProto, IntoProto)]
pub struct PitchName {
    #[proto_enum]
    pub scale_value: ScaleValue,
    pub octave: Octave,
}

impl From<PitchName> for PitchValue {
    fn from(val: PitchName) -> Self {
        let scale_value_pitch: PitchValue = val.scale_value.into();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_name_pitch_value_round_trip () {
        let pitch_name = PitchName {
            scale_value: ScaleValue::C,
            octave: 4
        };
        let pitch_value: PitchValue = pitch_name.into();
        let result: PitchName = pitch_value.into();
        assert_eq!(pitch_name, result);
    }

    #[test]
    fn pitch_name_addition () {
        let pitch_name = PitchName {
            scale_value: ScaleValue::C,
            octave: 4
        };
        let pitch_delta: PitchValue = 5;
        let result = pitch_name + pitch_delta;
        let target_result = PitchName {
            scale_value: ScaleValue::F,
            octave: 4
        };
        assert_eq!(target_result, result);
    }
}
