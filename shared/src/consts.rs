use crate::model::{PitchName, ScaleValue};
use crate::types::Freq;
use lazy_static::lazy_static;
use std::net::SocketAddr;

pub struct ReferencePitch<'a> {
    pub pitch_name: &'a PitchName,
    pub frequency: Freq,
}

pub const REFERENCE_PITCH: ReferencePitch<'static> = ReferencePitch {
    pitch_name: &PitchName {
        scale_value: ScaleValue::A,
        octave: 4,
    },
    frequency: 440.0,
};

lazy_static! {
    // The frequency multiplier for a semitone.
    pub static ref SEMITONE_FREQ: f32 = 2.0_f32.powf(1.0 / 12.0);
}

pub const XERIC_URL: &str = "http://127.0.0.1:3000";
pub const HYDRIC_URL: &str = "http://127.0.0.1:8080";

lazy_static! {
    pub static ref XERIC_SOCKET_ADDR: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 3000));
}
