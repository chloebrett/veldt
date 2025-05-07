use super::{Generator, Oscillator};
use crate::model::{AdsrEnvelope, EffectInstance, LfoConfig, ModMatrix};
use crate::pmodel::SubSynthConfigProto;
use crate::serialize::map_vec;
use local_macro::IntoProto;

#[derive(Clone, Debug, PartialEq, IntoProto)]
pub struct SubSynthConfig {
    #[proto_repeated]
    pub oscillators: [Oscillator; 3],

    #[proto_repeated]
    pub envelopes: [AdsrEnvelope; 3],

    #[proto_repeated]
    pub lfos: [LfoConfig; 3],

    #[proto_optional]
    pub matrix: ModMatrix,

    #[proto_optional]
    pub filter: EffectInstance,
}

impl<'a> TryFrom<&'a Generator> for &'a SubSynthConfig {
    type Error = ();

    fn try_from(item: &'a Generator) -> Result<Self, ()> {
        match item {
            Generator::SubSynth(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a mut Generator> for &'a mut SubSynthConfig {
    type Error = ();

    fn try_from(item: &'a mut Generator) -> Result<Self, ()> {
        match item {
            Generator::SubSynth(it) => Ok(it),
            _ => Err(()),
        }
    }
}

impl From<SubSynthConfigProto> for SubSynthConfig {
    fn from(proto: SubSynthConfigProto) -> Self {
        let SubSynthConfigProto {
            oscillators,
            envelopes,
            lfos,
            matrix,
            filter,
        } = proto;

        SubSynthConfig {
            oscillators: map_vec(oscillators)
                .try_into()
                .expect("Expected 3 oscillators!"),
            envelopes: map_vec(envelopes)
                .try_into()
                .expect("Expected 3 envelopes!"),
            lfos: map_vec(lfos).try_into().expect("Expected 3 LFOs!"),
            matrix: matrix.unwrap().into(),
            filter: filter.unwrap().into(),
        }
    }
}
