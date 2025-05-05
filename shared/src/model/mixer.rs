use crate::model::EffectInstance;
use crate::pmodel::*;
use crate::types::Volume;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerChannel {
    pub volume: Volume,

    #[proto_repeated]
    pub effects: Vec<EffectInstance>,
}
