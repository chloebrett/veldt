use crate::pmodel::DelayConfigProto;
use crate::types::{Milliseconds, Volume};
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct DelayConfig {
    pub delay_ms: Milliseconds,
    pub feedback: Volume,
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            delay_ms: 100.0,
            feedback: 0.5,
        }
    }
}
