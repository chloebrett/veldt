use crate::model::scale_value::ScaleValue;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PitchName {
    pub scale_value: ScaleValue,
    pub octave: Octave
}
