use crate::model::EffectInstance;
use crate::pmodel::*;
use crate::types::Volume;
use local_macro::{FromProto, IntoProto};

#[derive(Default, Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct Mixer {
    #[proto_optional]
    pub matrix: MixerMatrix,

    #[proto_repeated]
    pub channels: Vec<MixerChannel>,
}

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerChannel {
    pub volume: Volume,

    #[proto_repeated]
    pub effects: Vec<EffectInstance>,
}

/// Note: distinct from ModMatrix because the mixer's matrix is somewhat symmetrical -
/// that is, inputs and outputs come from the same set (mixer channels) and therefore we
/// need special logic to avoid graph cycles.
#[derive(Clone, Default, Debug, PartialEq, FromProto, IntoProto)]
pub struct MixerMatrix {
    #[proto_type_u8]
    pub rows: u8,

    #[proto_type_u8]
    pub cols: u8,

    #[proto_repeated]
    pub matrix: Vec<f32>,
}

impl MixerMatrix {
    pub fn with_channels(count: u8) -> Self {
        MixerMatrix {
            rows: count,
            cols: count,
            matrix: vec![0.0; count as usize * count as usize],
        }
    }
}
