use super::{Generator, Oscillator};
use crate::model::{AdsrEnvelope, EqConfig, EqType, LfoConfig, ModMatrix, WaveType};
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
    pub lpf: EqConfig,
}

const BASE_OSC: Oscillator = Oscillator {
    wave: WaveType::Sine,
    volume: 1.0,
    pan: 0.0,
    osc_detune: 0.0,
    osc_count: 1,
    unison_detune: 0.0,
};

const BASE_LFO: LfoConfig = LfoConfig {
    wave: WaveType::Sine,
    frequency: 1.0,
};

const BASE_ENV: AdsrEnvelope = AdsrEnvelope {
    attack: 100.0,
    decay: 100.0,
    sustain: 0.8,
    release: 100.0,
};

const BASE_LPF: EqConfig = EqConfig {
    kind: EqType::SimpleSecondOrderLowPass,
    fc: 1000.0,
    gain: 0.0,
    q: 1.0,
};

impl Default for SubSynthConfig {
    fn default() -> Self {
        Self {
            oscillators: [
                Oscillator {
                    wave: WaveType::Sine,
                    ..BASE_OSC
                },
                Oscillator {
                    wave: WaveType::Triangle,
                    ..BASE_OSC
                },
                Oscillator {
                    wave: WaveType::Square,
                    ..BASE_OSC
                },
            ],
            lfos: [BASE_LFO; 3],
            envelopes: [BASE_ENV; 3],
            matrix: ModMatrix::new(6, 4),
            lpf: BASE_LPF,
        }
    }
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
            lpf,
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
            lpf: lpf.unwrap().into(),
        }
    }
}
