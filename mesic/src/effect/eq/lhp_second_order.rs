use super::low_high::LowHigh;
use super::filter::second_order;
use std::f32::consts::TAU;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;

pub fn lhp_second_order(config: EqConfig, input: Vec<f32>, low_high: LowHigh) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let theta: f32 = TAU * config.fc / fs;
    let d: f32 = 1.0 / config.q;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p272
    let alpha = 0.5 * d * theta.sin();
    let beta: f32 = 0.5 * (1.0 - alpha) / (1.0 + alpha);
    let gamma = (0.5 + beta) * theta.cos();
    let a1 = match low_high {
        LowHigh::Low => 0.5 * (0.5 + beta - gamma),
        LowHigh::High => 0.5 * (0.5 + beta + gamma),
    };
    let a0 = 0.5 * a1;
    let a2 = a0;
    let b1 = -2.0 * gamma;
    let b2 = 2.0 * beta;

    second_order(input, a0, a1, a2, b1, b2)
}
