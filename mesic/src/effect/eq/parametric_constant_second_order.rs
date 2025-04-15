use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn parametric_constant_q(config: &EqConfig) -> SecondOrderFilter {
    let fs = SAMPLE_RATE as f32;
    let fc = config.fc;
    let q = config.q;

    let k: f32 = (PI * fc / fs).tan();
    let v0: f32 = 10.0_f32.powf(config.gain / 20.0);

    let k2 = k * k;

    let d0: f32 = 1.0 / (1.0 + (1.0 / q) * k + k2);
    let e0: f32 = 1.0 / (1.0 + (1.0 / (v0 * q)) * k + k2);

    let alpha: f32 = 1.0 + (v0 / q) * k + k2;
    let beta: f32 = 2.0 * (k2 - 1.0);
    let gamma: f32 = 1.0 - (v0 / q) * k + k2;
    let delta: f32 = 1.0 - (1.0 / q) * k + k2;
    let eta = 1.0 - (1.0 / (v0 * q)) * k + k2;

    let boost = config.gain >= 0.0;

    let (a0, a1, a2, b1, b2) = if boost {
        (alpha * d0, beta * d0, gamma * d0, beta * d0, delta * d0)
    } else {
        (d0 * e0, beta * e0, delta * e0, beta * e0, eta * e0)
    };

    SecondOrderFilter::new_wet(
        SecondOrderFilterConfig { a0, a1, a2, b1, b2 }
    )
}
