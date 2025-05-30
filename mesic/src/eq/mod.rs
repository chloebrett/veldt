mod apf_first_order;
mod apf_second_order;
mod bps_basic;
pub mod eq_display;
mod filter;
mod first_order_all_pole;
mod lhp_first_order;
mod lhp_second_order;
mod lhp_second_order_lr;
mod low_high;
mod parametric_constant_second_order;
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
use parametric_constant_second_order::*;
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
        EqType::ParametricSecondOrderConstantQ => Box::new(parametric_constant_q(config)),
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

pub fn adjusted_eq_config_for_lfo(config: &EqConfig, lfo_state: f32) -> EqConfig {
    let min_freq = 20.0;
    let max_freq = 20000.0;

    // Make sure the lfo_cutoff is in the range of -1.0 to 1.0
    // This implementation of the LFO-LPF relation is based on the the ableton synth version
    // https://learningsynths.ableton.com/en/playground
    let new_cutoff = config.fc + lfo_state * (max_freq - min_freq); // At 1.0 the LFO should go all the way to max_freq
    let new_cutoff = new_cutoff.clamp(min_freq, max_freq);

    let mut adjusted_config = config.clone();
    adjusted_config.fc = new_cutoff;
    adjusted_config
}
