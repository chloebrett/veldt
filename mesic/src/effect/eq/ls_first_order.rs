use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn ls_first_order(config: &EqConfig) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p278 (Low Shelving)
    let thetac: f32 = PI * config.fc / fs;
    let mu: f32 = 10.0_f32.powf(config.gain / 20.0);
    let beta: f32 = 4.0 / (1.0 + mu);
    let delta: f32 = beta * (thetac * 0.5).tan();
    let gamma: f32 = (1.0 - delta) / (1.0 + delta);

    let frac: f32 = 0.5 * (1.0 - gamma);

    let a0: f32 = frac;
    let a1: f32 = frac;
    let a2: f32 = 0.0;
    let b1: f32 = -gamma;
    let b2: f32 = 0.0;
    // let c0: f32 = mu - 1.0;
    // let d0: f32 = 1.0;

    SecondOrderFilter::new(SecondOrderFilterConfig { a0, a1, a2, b1, b2 })
}
