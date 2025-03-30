use super::filter::FirstOrderFilter;
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::TAU;

/// Simple first order low/high pass.
/// Ignores Q value - this is the equivalent of the second order l/h p with Q = 0.707.
pub fn lhp_first_order(config: &EqConfig, low_high: LowHigh) -> FirstOrderFilter {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * config.fc / fs;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p271
    let gamma: f32 = theta.cos() / (1.0 + theta.sin());
    let a0: f32 = match low_high {
        LowHigh::Low => 0.5 * (1.0 - gamma),
        LowHigh::High => 0.5 * (1.0 + gamma),
    };
    let a1: f32 = a0;
    let b1: f32 = -gamma;

    FirstOrderFilter::new(a0, a1, b1)
}
