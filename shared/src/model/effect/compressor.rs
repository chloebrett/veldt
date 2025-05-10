use crate::pmodel::CompressorConfigProto;
use crate::types::{KnobPosition, Milliseconds, Volume};
use local_macro::{FromProto, IntoProto};

#[derive(Clone, Debug, PartialEq, FromProto, IntoProto)]
pub struct CompressorConfig {
    // TODO: use Decibels instead of Volume.
    pub threshold: Volume,
    pub attack_ms: Milliseconds,
    pub release_ms: Milliseconds,
    pub ratio: KnobPosition,
    pub gain: Volume,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            threshold: -10.0,
            attack_ms: 30.0,
            release_ms: 30.0,
            ratio: 1.5,
            gain: 1.0,
        }
    }
}

#[cfg(test)]
// Test effect configs as they may not appear in the Project test.
mod tests {
    use super::*;
    use crate::testing::proto::proto_testing::assert_proto_round_trip;

    #[test]
    fn compressor_config_proto_round_trip() {
        let compressor_config = CompressorConfig {
            threshold: 0.3,
            attack_ms: 0.2,
            release_ms: 0.6,
            ratio: 0.2,
            gain: 1.0,
        };
        assert_proto_round_trip::<CompressorConfig, CompressorConfigProto>(compressor_config);
    }
}
