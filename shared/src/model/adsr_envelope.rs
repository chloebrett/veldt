use crate::pmodel::*;
use crate::types::*;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct AdsrEnvelope {
    pub attack: Beats,

    pub decay: Beats,

    pub sustain: Volume,

    pub release: Beats,
}
