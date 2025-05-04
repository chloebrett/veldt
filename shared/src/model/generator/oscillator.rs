use crate::model::WaveType;
use crate::pmodel::OscillatorProto;
use crate::types::{KnobPosition, Volume};
use local_macro::{FromProto, IntoProto};


#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Oscillator {
    #[proto_enum]
    pub wave: WaveType,

    pub volume: Volume,

    pub pan: KnobPosition,

    pub osc_detune: f32,

    pub osc_count: u32,

    pub unison_detune: f32,
}
