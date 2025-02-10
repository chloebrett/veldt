use crate::pmodel::*;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AdsrEnvelope {
    pub attack: Beats,

    pub decay: Beats,

    pub sustain: Volume,

    pub release: Beats,
}

impl From<AdsrEnvelopeProto> for AdsrEnvelope {
    fn from(item: AdsrEnvelopeProto) -> Self {
        AdsrEnvelope {
            attack: item.attack,
            decay: item.decay,
            sustain: item.sustain,
            release: item.release,
        }
    }
}

impl From<AdsrEnvelope> for AdsrEnvelopeProto {
    fn from(item: AdsrEnvelope) -> Self {
        AdsrEnvelopeProto {
            attack: item.attack,
            decay: item.decay,
            sustain: item.sustain,
            release: item.release,
        }
    }
}
