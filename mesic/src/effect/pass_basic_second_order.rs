use super::filter::second_degree_filter;
use super::low_high::LowHigh;
use crate::consts::SAMPLE_RATE;
use shared::types::{Freq, KnobPosition};
use std::f32::consts::{PI, TAU};

pub fn apply_low_high_pass_basic_second_order(
    dry_signal: Vec<f32>,
    fc: Freq,
    q_value: KnobPosition,
    low_high: LowHigh,
) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let theta: f32 = TAU * fc / fs;
    let d: f32 = 1.0 / q_value;

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

    second_degree_filter(dry_signal, a0, a1, a2, b1, b2)
}

pub fn apply_band_pass_basic(dry_signal: Vec<f32>, fc: Freq, q: KnobPosition) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let k = (PI * fc / fs).tan();
    let k2 = k * k;
    let k2_q = k2 * q;
    let delta = k2_q + k + q;
    let delta_recip = 1.0 / delta;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p273
    let a0: f32 = k * delta_recip;
    let a1: f32 = 0.0;
    let a2: f32 = -a0;
    let b1: f32 = 2.0 * (k2_q - q) * delta_recip;
    let b2: f32 = (k2_q - k + q) * delta_recip;

    second_degree_filter(dry_signal, a0, a1, a2, b1, b2)
}

pub fn apply_band_stop_basic(dry_signal: Vec<f32>, fc: Freq, q: KnobPosition) -> Vec<f32> {
    let fs = SAMPLE_RATE as f32;
    let k = (PI * fc / fs).tan();
    let k2 = k * k;
    let k2_q = k2 * q;
    let delta = k2_q + k + q;
    let delta_recip = 1.0 / delta;

    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p273
    let a0: f32 = (k2_q + q) * delta_recip;
    let a1: f32 = 2.0 * (k2_q - q) * delta_recip;
    let a2: f32 = a0;
    let b1: f32 = a1;
    let b2: f32 = (k2_q - k + q) * delta_recip;

    second_degree_filter(dry_signal, a0, a1, a2, b1, b2)
}
