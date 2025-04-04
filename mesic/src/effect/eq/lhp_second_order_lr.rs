use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn lhp_second_order_lr(config: &EqConfig, low_high: LowHigh) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;
    
    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p275 (Linkwitz-Riley)
    let theta: f32 = PI * config.fc / fs;
    let omega: f32 = PI * config.fc;
    let kappa: f32 = omega / theta.tan();
    let delta: f32 = kappa.powi(2) + omega.powi(2) + (2.0 * kappa * omega);
    let a0: f32 = match low_high {
        LowHigh::Low => omega.powi(2) / delta,
        LowHigh::High => kappa.powi(2) / delta,
    };
    let a1: f32 = match low_high {
        LowHigh::Low => 2.0 * (omega.powi(2) / delta),
        LowHigh::High => -2.0 * (kappa.powi(2) / delta),
    };
    
    let a2: f32 = match low_high {
        LowHigh::Low => omega.powi(2) / delta,
        LowHigh::High => kappa.powi(2) / delta,
    };
    let b1: f32 = (-2.0 * kappa.powi(2) + 2.0 * omega.powi(2)) / delta;
    let b2: f32 = (-2.0 * kappa * omega + kappa.powi(2) + omega.powi(2)) / delta;

    SecondOrderFilter::new(SecondOrderFilterConfig { a0, a1, a2, b1, b2 })
}
