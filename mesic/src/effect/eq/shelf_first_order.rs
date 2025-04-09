use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn shelf_first_order(config: &EqConfig, low_high: LowHigh) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p278 (Shelving Filters)
    let thetac: f32 = PI * config.fc / fs;
    let mu: f32 = 10.0_f32.powf(config.gain / 20.0);
    let beta: f32 = match low_high {
        LowHigh::Low => 4.0 / (1.0 + mu),
        LowHigh::High => (1.0 + mu) / 4.0,
    };
    let delta: f32 = beta * (thetac * 0.5).tan();
    let gamma: f32 = (1.0 - delta) / (1.0 + delta);

    let low_frac: f32 = 0.5 * (1.0 - gamma);
    let high_frac: f32 = 0.5 * (1.0 + gamma);

    let a0: f32 = match low_high {
        LowHigh::Low => low_frac,
        LowHigh::High => high_frac,
    };
    let a1: f32 = match low_high {
        LowHigh::Low => low_frac,
        LowHigh::High => -high_frac,
    };
    let a2: f32 = 0.0;
    let b1: f32 = -gamma;
    let b2: f32 = 0.0;
    // let c0: f32 = mu - 1.0;
    // let d0: f32 = 1.0;

    SecondOrderFilter::new(SecondOrderFilterConfig { a0, a1, a2, b1, b2 })
}
