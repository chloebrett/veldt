use crate::model::WaveType;
use crate::pmodel::ModDelayConfigProto;
use crate::types::Freq;
use local_macro::{FromProto, IntoProto};

/// Modulated delay, e.g. vibrato, flanger, chorus.
#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct ModDelayConfig {
    pub min_depth: u32, // Samples
    pub max_depth: u32, // Samples
    pub freq: Freq,     // LFO rate

    #[proto_enum]
    pub lfo_type: WaveType,
    // No support for feedback for now.
}

impl Default for ModDelayConfig {
    fn default() -> Self {
        Self {
            min_depth: 100,
            max_depth: 300,
            freq: 10.0,
            lfo_type: WaveType::Triangle,
        }
    }
}
