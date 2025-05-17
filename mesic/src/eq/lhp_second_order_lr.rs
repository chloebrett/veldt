use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn lhp_second_order_lr(config: &EqConfig, low_high: LowHigh) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p275 (Linkwitz-Riley)
    let theta = PI * config.fc / fs;
    let omega = PI * config.fc;
    let omega2 = omega.powi(2);
    let kappa = omega / theta.tan();
    let kappa2 = kappa.powi(2);
    let delta_reciprocal = 1.0 / (kappa2 + omega2 + (2.0 * kappa * omega));
    let a0 = match low_high {
        LowHigh::Low => omega2 * delta_reciprocal,
        LowHigh::High => kappa2 * delta_reciprocal,
    };
    let a1 = match low_high {
        LowHigh::Low => 2.0 * a0,
        LowHigh::High => -2.0 * a0,
    };

    let a2 = a0;
    let b1 = (2.0 * (omega2 - kappa2)) * delta_reciprocal;
    let b2 = (-2.0 * kappa * omega + kappa2 + omega2) * delta_reciprocal;

    SecondOrderFilter::new_wet(SecondOrderFilterConfig { a0, a1, a2, b1, b2 })
}
