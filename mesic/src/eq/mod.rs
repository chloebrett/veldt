mod apf_first_order;
mod apf_second_order;
mod bps_basic;
pub mod eq_display;
pub mod filter;
mod first_order_all_pole;
mod lhp_first_order;
mod lhp_second_order;
mod lhp_second_order_lr;
mod low_high;
mod parametric_constant_second_order;
mod parametric_second_order;
pub mod pink;
mod resonator_sa;
mod resonator_simple;
mod shelf_first_order;

use apf_first_order::*;
use apf_second_order::*;
use bps_basic::*;
use filter::Filter;
use first_order_all_pole::*;
use lhp_first_order::*;
use lhp_second_order::*;
use lhp_second_order_lr::*;
use low_high::LowHigh;
use parametric_constant_second_order::*;
use parametric_second_order::*;
use resonator_sa::*;
use resonator_simple::*;
use shared::model::{EqConfig, EqType};
use shelf_first_order::*;

pub fn eq_filter(config: &EqConfig) -> Filter {
    match config.kind {
        EqType::SimpleResonator => resonator_simple(config),
        EqType::SmithAngellResonator => resonator_smith_angell(config),
        EqType::SimpleFirstOrderLowPass => lhp_first_order(config, LowHigh::Low),
        EqType::SimpleFirstOrderHighPass => lhp_first_order(config, LowHigh::High),
        EqType::SimpleSecondOrderLowPass => lhp_second_order(config, LowHigh::Low),
        EqType::SimpleSecondOrderHighPass => lhp_second_order(config, LowHigh::High),
        EqType::SimpleSecondOrderResonator => band_pass_basic(config),
        EqType::FirstOrderAllPass => apf_first_order(config),
        EqType::SecondOrderAllPass => apf_second_order(config),
        EqType::SimpleSecondOrderBandStop => band_stop_basic(config),
        EqType::LinkwitzRileySecondOrderLowPass => lhp_second_order_lr(config, LowHigh::Low),
        EqType::LinkwitzRileySecondOrderHighPass => lhp_second_order_lr(config, LowHigh::High),
        EqType::ParametricSecondOrderNonConstantQ => parametric_non_constant_q(config),
        EqType::FirstOrderAllPole => first_order_all_pole(config),
        EqType::LowShelvingFirstOrder => shelf_first_order(config, LowHigh::Low),
        EqType::HighShelvingFirstOrder => shelf_first_order(config, LowHigh::High),
        EqType::ParametricSecondOrderConstantQ => parametric_constant_q(config),
    }
}

pub struct BiquadCoefficients {
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub b1: f32,
    pub b2: f32,
    pub wet: f32,
    pub dry: f32,
}

/// This function gets just the biquad coefficients of each EQ filter which is needed for visualising the EQ wave.
pub fn get_eq_filter_coeffs(config: &EqConfig) -> Option<BiquadCoefficients> {
    match config.kind {
        EqType::SimpleSecondOrderLowPass
        | EqType::SimpleSecondOrderHighPass
        | EqType::LinkwitzRileySecondOrderLowPass
        | EqType::LinkwitzRileySecondOrderHighPass
        | EqType::SimpleSecondOrderResonator
        | EqType::SecondOrderAllPass
        | EqType::SimpleSecondOrderBandStop
        | EqType::ParametricSecondOrderConstantQ
        | EqType::ParametricSecondOrderNonConstantQ
        | EqType::SimpleResonator
        | EqType::SmithAngellResonator => {
            let filter = match config.kind {
                EqType::SimpleSecondOrderLowPass => lhp_second_order(config, LowHigh::Low),
                EqType::SimpleSecondOrderHighPass => lhp_second_order(config, LowHigh::High),
                EqType::LinkwitzRileySecondOrderLowPass => {
                    lhp_second_order_lr(config, LowHigh::Low)
                }
                EqType::LinkwitzRileySecondOrderHighPass => {
                    lhp_second_order_lr(config, LowHigh::High)
                }
                EqType::SimpleSecondOrderResonator => band_pass_basic(config),
                EqType::SecondOrderAllPass => apf_second_order(config),
                EqType::SimpleSecondOrderBandStop => band_stop_basic(config),
                EqType::ParametricSecondOrderConstantQ => parametric_constant_q(config),
                EqType::ParametricSecondOrderNonConstantQ => parametric_non_constant_q(config),
                EqType::SimpleResonator => resonator_simple(config),
                EqType::SmithAngellResonator => resonator_smith_angell(config),
                _ => unreachable!("Handled in outer match"),
            };
            let coeffs_config = filter.config;
            let (wet, dry) = if let Some(mix) = filter.mix {
                (mix.wet, mix.dry)
            } else {
                (1.0, 0.0) // if absent assume wet = 1.0 and dry = 0.0
            };
            Some(BiquadCoefficients {
                a0: coeffs_config.a0,
                a1: coeffs_config.a1,
                a2: coeffs_config.a2,
                b1: coeffs_config.b1,
                b2: coeffs_config.b2,
                wet,
                dry,
            })
        }
        EqType::SimpleFirstOrderLowPass
        | EqType::SimpleFirstOrderHighPass
        | EqType::FirstOrderAllPass
        | EqType::FirstOrderAllPole
        | EqType::LowShelvingFirstOrder
        | EqType::HighShelvingFirstOrder => {
            let filter = match config.kind {
                EqType::SimpleFirstOrderLowPass => lhp_first_order(config, LowHigh::Low),
                EqType::SimpleFirstOrderHighPass => lhp_first_order(config, LowHigh::High),
                EqType::FirstOrderAllPass => apf_first_order(config),
                EqType::FirstOrderAllPole => first_order_all_pole(config),
                EqType::LowShelvingFirstOrder => shelf_first_order(config, LowHigh::Low),
                EqType::HighShelvingFirstOrder => shelf_first_order(config, LowHigh::High),
                _ => unreachable!("Handled in outer match"),
            };
            let coeffs_config = filter.config;
            let (wet, dry) = if let Some(mix) = filter.mix {
                (mix.wet, mix.dry)
            } else {
                (1.0, 0.0) // if absent assume wet = 1.0 and dry = 0.0
            };
            Some(BiquadCoefficients {
                a0: coeffs_config.a0,
                a1: coeffs_config.a1,
                a2: 0.0, // Set a2 to zero for first order filters
                b1: coeffs_config.b1,
                b2: 0.0, // Set b2 to zero for first order filters
                wet,
                dry,
            })
        }
    }
}
