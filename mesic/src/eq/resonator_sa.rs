use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::TAU;

pub fn resonator_smith_angell(config: &EqConfig) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;
    let theta = TAU * config.fc / fs;
    let bandwidth = config.fc / config.q;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p260
    let b2 = (-TAU * bandwidth / fs).exp();
    let b1 = (-4.0 * b2) / (1.0 + b2) * theta.cos();
    let a0 = 1.0 - b2.sqrt();
    let a1 = 0.0;
    let a2 = -a0;

    SecondOrderFilter::new_wet(SecondOrderFilterConfig { a0, a1, a2, b1, b2 })
}
