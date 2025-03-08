mod bps_basic;
mod filter;
mod lhp_first_order;
mod lhp_second_order;
mod low_high;
mod resonator_sa;
mod resonator_simple;

use bps_basic::*;
use lhp_first_order::*;
use lhp_second_order::*;
use resonator_simple::*;
use resonator_sa::*;
use low_high::LowHigh;
use shared::model::{EqConfig, EqType};
use super::EffectFilter;

impl EffectFilter for EqConfig {
    fn apply(self: EqConfig, input: Vec<f32>) -> Vec<f32> {
        match self.kind {
            EqType::SimpleResonator => resonator_simple(self, input),
            EqType::SmithAngellResonator => resonator_smith_angell(self, input),
            EqType::SimpleFirstOrderLowPass => lhp_first_order(self, input, LowHigh::Low),
            EqType::SimpleFirstOrderHighPass => lhp_first_order(self, input, LowHigh::High),
            EqType::SimpleSecondOrderLowPass => lhp_second_order(self, input, LowHigh::Low),
            EqType::SimpleSecondOrderHighPass => lhp_second_order(self, input, LowHigh::High),
            EqType::SimpleSecondOrderResonator => band_pass_basic(self, input),
            EqType::SimpleSecondOrderBandStop => band_stop_basic(self, input),
            _ => panic!("Eq type not implemented!"),
        }
    }
}
