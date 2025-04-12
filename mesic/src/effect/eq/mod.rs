mod apf_first_order;
mod apf_second_order;
mod bps_basic;
mod filter;
mod first_order_all_pole;
mod lhp_first_order;
mod lhp_second_order;
mod lhp_second_order_lr;
mod low_high;
mod parametric_second_order;
mod resonator_sa;
mod resonator_simple;
mod shelf_first_order;

use apf_first_order::*;
use apf_second_order::*;
use bps_basic::*;
use dasp_graph::Buffer;
use first_order_all_pole::*;
use lhp_first_order::*;
use lhp_second_order::*;
use lhp_second_order_lr::*;
use low_high::LowHigh;
use parametric_second_order::*;
use resonator_sa::*;
use resonator_simple::*;
use shared::model::{EqConfig, EqType};
use shelf_first_order::*;

pub trait ApplyFilter {
    fn apply(&mut self, buffer: &mut Buffer);
}

pub fn eq_filter(config: &EqConfig) -> Box<dyn ApplyFilter + Send> {
    match config.kind {
        EqType::SimpleResonator => Box::new(resonator_simple(config)),
        EqType::SmithAngellResonator => Box::new(resonator_smith_angell(config)),
        EqType::SimpleFirstOrderLowPass => Box::new(lhp_first_order(config, LowHigh::Low)),
        EqType::SimpleFirstOrderHighPass => Box::new(lhp_first_order(config, LowHigh::High)),
        EqType::SimpleSecondOrderLowPass => Box::new(lhp_second_order(config, LowHigh::Low)),
        EqType::SimpleSecondOrderHighPass => Box::new(lhp_second_order(config, LowHigh::High)),
        EqType::SimpleSecondOrderResonator => Box::new(band_pass_basic(config)),
        EqType::FirstOrderAllPass => Box::new(apf_first_order(config)),
        EqType::SecondOrderAllPass => Box::new(apf_second_order(config)),
        EqType::SimpleSecondOrderBandStop => Box::new(band_stop_basic(config)),
        EqType::LinkwitzRileySecondOrderLowPass => {
            Box::new(lhp_second_order_lr(config, LowHigh::Low))
        }
        EqType::LinkwitzRileySecondOrderHighPass => {
            Box::new(lhp_second_order_lr(config, LowHigh::High))
        }
        EqType::ParametricSecondOrderNonConstantQ => Box::new(parametric_non_constant_q(config)),
        EqType::FirstOrderAllPole => Box::new(first_order_all_pole(config)),
        EqType::LowShelvingFirstOrder => Box::new(shelf_first_order(config, LowHigh::Low)),
        EqType::HighShelvingFirstOrder => Box::new(shelf_first_order(config, LowHigh::High)),
    }
}
