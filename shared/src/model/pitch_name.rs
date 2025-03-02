use std::fmt;
use crate::pmodel::*;
use crate::model::scale_value::ScaleValue;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PitchName {
    pub scale_value: ScaleValue,
    pub octave: Octave
}

impl Into<PitchValue> for PitchName {
    fn into(self: Self) -> PitchValue {
        let scale_value_pitch = self.scale_value as PitchValue;
        let octave_pitch = self.octave * 12;
        scale_value_pitch + octave_pitch
    }
}

impl From<PitchValue> for PitchName {
    fn from(pitch_value: PitchValue) -> Self {
        Self {
            scale_value: ScaleValue::from(pitch_value),
            octave: pitch_value / 12  
        }
    }
}

impl fmt::Display for PitchName {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f, "{}{}", self.scale_value, self.octave)
   } 
}

impl From<PitchNameProto> for PitchName {
    fn from(item: PitchNameProto) -> Self {
        PitchName {
            scale_value: TryInto::<ScaleValueProto>::try_into(item.scale_value)
                .unwrap()
                .into(),
            octave: item.octave
        }
    }
}

impl From<PitchName> for PitchNameProto {
    fn from(item: PitchName) -> Self {
        PitchNameProto {
            scale_value: Into::<ScaleValueProto>::into(item.scale_value) as i32,
            octave: item.octave,
        }
    }
}
