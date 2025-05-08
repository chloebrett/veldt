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
            attack: 30.0,
            decay: 20.0,
            sustain: 0.8,
            release: 25.0,
        }
    }
}
