use crate::model::adsr_envelope::AdsrEnvelope;
use crate::model::wave_type::WaveType;
use crate::pmodel::*;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Synth {
    pub wave: WaveType,

    pub envelope: AdsrEnvelope,

    pub volume: Volume,
}

impl From<SynthProto> for Synth {
    fn from(item: SynthProto) -> Self {
        Synth {
            wave: TryInto::<WaveTypeProto>::try_into(item.wave)
                .unwrap()
                .into(),
            envelope: item.envelope.unwrap().into(),
            volume: item.volume,
        }
    }
}

impl From<Synth> for SynthProto {
    fn from(item: Synth) -> Self {
        SynthProto {
            wave: Into::<WaveTypeProto>::into(item.wave) as i32,
            envelope: Some(item.envelope.into()),
            volume: item.volume,
        }
    }
}
