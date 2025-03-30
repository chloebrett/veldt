use super::pitch_name::PitchName;
use crate::pmodel::*;
use crate::types::*;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Note {
    #[proto_optional]
    pub pitch_name: PitchName,
    pub beats: Beats,
}
