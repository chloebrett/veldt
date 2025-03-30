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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_adsr_envelope_to_proto_and_back() {
        let envelope = AdsrEnvelope {
            attack: 0.3,
            decay: 0.2,
            sustain: 0.4,
            release: 1.2,
        };
        let proto: AdsrEnvelopeProto = envelope.clone().into();
        let result: AdsrEnvelope = proto.into();
        assert_eq!(envelope, result);
    }
}
