mod bps_basic;
mod filter;
mod lhp_first_order;
mod lhp_second_order;
mod low_high;
mod resonator_sa;
mod resonator_simple;

use super::ApplyEffect;
use bps_basic::*;
use lhp_first_order::*;
use lhp_second_order::*;
use low_high::LowHigh;
use resonator_sa::*;
use resonator_simple::*;
use shared::model::{EqConfig, EqType};

impl ApplyEffect for EqConfig {
    fn apply(&self, input: &Vec<f32>) -> Vec<f32> {
        let filter: Box<dyn ApplyEffect> = match self.kind {
            EqType::SimpleResonator => Box::new(resonator_simple(self)),
            EqType::SmithAngellResonator => Box::new(resonator_smith_angell(self)),
            EqType::SimpleFirstOrderLowPass => Box::new(lhp_first_order(self, LowHigh::Low)),
            EqType::SimpleFirstOrderHighPass => Box::new(lhp_first_order(self, LowHigh::High)),
            EqType::SimpleSecondOrderLowPass => Box::new(lhp_second_order(self, LowHigh::Low)),
            EqType::SimpleSecondOrderHighPass => Box::new(lhp_second_order(self, LowHigh::High)),
            EqType::SimpleSecondOrderResonator => Box::new(band_pass_basic(self)),
            EqType::SimpleSecondOrderBandStop => Box::new(band_stop_basic(self)),
            _ => panic!("EQ type not implemented!"),
        };

        // TODO: cache filters so that they can process multiple inputs before needing to be
        // recreated.
        filter.apply(input)
    }
}
