use crate::pmodel::*;
use crate::types::*;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct AdsrEnvelope {
    pub attack: Milliseconds,

    pub decay: Milliseconds,

    pub sustain: Volume,

    pub release: Milliseconds,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            attack: 100.0,
            decay: 100.0,
            sustain: 0.8,
            release: 100.0,
        }
    }
}
