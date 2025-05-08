use crate::{
    model::EqType,
    pmodel::SubSynthLpfProto,
    types::{Freq, GainDB, KnobPosition},
};
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct SubSynthLpf {
    #[proto_enum]
    pub kind: EqType,

    pub fc: Freq,

    pub gain: GainDB,

    pub q: KnobPosition,
}

impl Default for SubSynthLpf {
    fn default() -> Self {
        Self {
            kind: EqType::SimpleSecondOrderLowPass,
            fc: 1000.0,
            gain: 0.0,
            q: 1.0,
        }
    }
}
