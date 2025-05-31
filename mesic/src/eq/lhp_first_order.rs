use super::filter::{Filter, FilterConfig};
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::TAU;

/// Simple first order low/high pass.
/// Ignores Q value - this is the equivalent of the second order l/h p with Q = 0.707.
pub fn lhp_first_order(config: &EqConfig, low_high: LowHigh) -> Filter {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * config.fc / fs;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p271
    let gamma = theta.cos() / (1.0 + theta.sin());
    let a0 = match low_high {
        LowHigh::Low => 0.5 * (1.0 - gamma),
        LowHigh::High => 0.5 * (1.0 + gamma),
    };
    let a1 = match low_high {
        LowHigh::Low => a0,
        LowHigh::High => -a0,
    };
    let b1 = -gamma;

    Filter::new_wet(FilterConfig {
        a0,
        a1,
        a2: 0.0,
        b1,
        b2: 0.0,
    })
}
