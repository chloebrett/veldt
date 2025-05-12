mod apf_first_order;
mod apf_second_order;
mod bps_basic;
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
use crate::SAMPLE_RATE;
use std::f32::consts::TAU;

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
}

/// This function gets just the biquad coefficients of each EQ filter which is needed for visualising the EQ wave.
pub fn get_eq_filter_coeffs(config: &EqConfig) -> Option<BiquadCoefficients> {
    match config.kind {
        EqType::SimpleSecondOrderLowPass | EqType::SimpleSecondOrderHighPass | EqType::LinkwitzRileySecondOrderLowPass | EqType::LinkwitzRileySecondOrderHighPass | EqType::SimpleSecondOrderResonator | EqType::SecondOrderAllPass | EqType::SimpleSecondOrderBandStop | EqType::ParametricSecondOrderConstantQ | EqType::ParametricSecondOrderNonConstantQ | EqType::SimpleResonator | EqType::SmithAngellResonator=> {
             let filter = match config.kind {
                 EqType::SimpleSecondOrderLowPass => lhp_second_order(config, LowHigh::Low),
                 EqType::SimpleSecondOrderHighPass => lhp_second_order(config, LowHigh::High),
                 EqType::LinkwitzRileySecondOrderLowPass => lhp_second_order_lr(config, LowHigh::Low),
                 EqType::LinkwitzRileySecondOrderHighPass => lhp_second_order_lr(config, LowHigh::High),
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
             Some(BiquadCoefficients {
                 a0: coeffs_config.a0,
                 a1: coeffs_config.a1,
                 a2: coeffs_config.a2,
                 b1: coeffs_config.b1,
                 b2: coeffs_config.b2,
             })
        }
        EqType::SimpleFirstOrderLowPass | EqType::SimpleFirstOrderHighPass | EqType::FirstOrderAllPass | EqType::FirstOrderAllPole | EqType::LowShelvingFirstOrder | EqType::HighShelvingFirstOrder => {
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
             Some(BiquadCoefficients {
                 a0: coeffs_config.a0,
                 a1: coeffs_config.a1,
                 a2: 0.0, // Set a2 to zero for first order filters
                 b1: coeffs_config.b1,
                 b2: 0.0, // Set b2 to zero for first order filters
             })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FrequencyResponsePoint {
    pub frequency: f32, // Hz
    pub gain_db: f32, // dB
}

// Used to generate frequency response curve for visualising EQ
// Essentially only need the magnitude of the H(z) formula as it represents the gain which is what gets plotted:
// H(z) = (a0 + a1*z^-1 + a2*z^-2)/(b0 + b1*z^-1 + b2*z^-2)
// To get the frequency response H(z) needs to be evaluated on the unit circle which means z needs to be replaced with e^(j*w*T)
// j is just i as in the imaginary number, w is the angualar frequency which is 2*pi*frequency, T is sampling period (1/SAMPLE_RATE)
// Computing H(e^(j*w*T)) directly is kind of messy so expand everything using Euler which will make it easier to get the numbers
// needed to calculate the magnitude which is all we're interested in anyway. See all the maths working here:
// https://drive.google.com/drive/folders/1flyEcv6HdQu_zPauZ6WSmkOsGYMAaVzx
pub fn calculate_frequency_response(
    coeffs: &BiquadCoefficients,
    num_points: usize,
    min_freq: f32,
    max_freq: f32,
) -> Vec<FrequencyResponsePoint> {
    let mut points = Vec::with_capacity(num_points);
    let sample_period = 1.0 / SAMPLE_RATE as f32;

    // Calculate frequencies on a logarithmic scale using log10 to space evenly
    let log_min_freq = min_freq.log10();
    let log_max_freq = max_freq.log10();
    let delta_log_freq = (log_max_freq - log_min_freq) / (num_points as f32 - 1.0);

    for i in 0..num_points {
        // Calculate the current frequency in Hz
        let current_log_freq = log_min_freq + (i as f32 * delta_log_freq);
        let freq_hz = 10.0_f32.powf(current_log_freq);

        // Calculate the angular frequency and omega * sample_period
        let w = TAU * freq_hz;
        let w_t = w * sample_period;

        // Calculate terms for the complex numerator and denominator
        let cos_w_t = w_t.cos();
        let sin_w_t = w_t.sin();
        let cos_2_w_t = (2.0 * w_t).cos();
        let sin_2_w_t = (2.0 * w_t).sin();

        // Numerator (N = RN + j*IN)
        let real_numerator = coeffs.a0 + coeffs.a1 * cos_w_t + coeffs.a2 * cos_2_w_t;
        let imaginary_numerator = -coeffs.a1 * sin_w_t - coeffs.a2 * sin_2_w_t;

        // Denominator (D = RD + j*ID)
        let real_denominator = 1.0 + coeffs.b1 * cos_w_t + coeffs.b2 * cos_2_w_t;
        let imaginary_denominator = -coeffs.b1 * sin_w_t - coeffs.b2 * sin_2_w_t;

        // Calculate the magnitude squared of the denominator: |D|^2 = RD^2 + ID^2
        let denominator_magnitude_sq = real_denominator * real_denominator + imaginary_denominator * imaginary_denominator;

        // To avoid accidentally dividing by zero just set magnitude to zero if the denominator magnitude is somehow zero or lower
        let magnitude;
        if denominator_magnitude_sq <= 0.0 {
             magnitude = 0.0;
        } else {
            let numerator_magnitude = (real_numerator * real_numerator + imaginary_numerator * imaginary_numerator).sqrt();
            let denominator_magnitude = denominator_magnitude_sq.sqrt();

            if denominator_magnitude == 0.0 {
                 magnitude = 0.0; // prevent division by zero
            } else {
                 magnitude = numerator_magnitude / denominator_magnitude;
            }
        }

        // Convert magnitude to decibels
        let gain_db = if magnitude > 0.0 {
            20.0 * magnitude.log10()
        } else {
            -100.0 // if magnitude is zero then just set the gain_db to something very low
        };

        points.push(FrequencyResponsePoint {
            frequency: freq_hz,
            gain_db,
        });
    }

    points
}