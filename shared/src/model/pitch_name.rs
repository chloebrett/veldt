use crate::pmodel::*;
use crate::model::scale_value::ScaleValue;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PitchName {
    pub scale_value: ScaleValue,
    pub octave: Octave
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
