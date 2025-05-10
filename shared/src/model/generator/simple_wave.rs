use super::AntiAliasingMode;
use crate::model::{AdsrEnvelope, PolyphonyMode, WaveType};
use crate::pmodel::SimpleWaveConfigProto;
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct SimpleWaveConfig {
    #[proto_enum]
    pub wave: WaveType,

    #[proto_optional]
    pub envelope: AdsrEnvelope,

    pub osc_count: u32,

    pub detune_cents: f32,

    #[proto_enum]
    pub anti_aliasing_mode: AntiAliasingMode,

    pub oversample_factor: u32,

    #[proto_enum]
    pub polyphony_mode: PolyphonyMode,

    pub polyphony_limit: u32,
}

impl Default for SimpleWaveConfig {
    fn default() -> Self {
        Self {
            wave: WaveType::Sine,
            envelope: AdsrEnvelope::default(),
            osc_count: 1,
            detune_cents: 0.0,
            anti_aliasing_mode: AntiAliasingMode::default(),
            oversample_factor: 1,
            polyphony_mode: PolyphonyMode::Polyphonic,
            polyphony_limit: 0,
        }
    }
}
