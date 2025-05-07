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

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            attack: 0.3,
            decay: 0.1,
            sustain: 0.8,
            release: 0.2,
        }
    }
}
