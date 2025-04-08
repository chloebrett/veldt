use super::filter::{FirstOrderFilter, FirstOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn first_order_all_pole(config: &EqConfig) -> FirstOrderFilter {
    // See "Designing Audio Effect Plugins in C++", W. Pirkle, p282
    let theta: f32 = 2.0 * PI * config.fc / SAMPLE_RATE as f32;
    let gamma: f32 = 2.0 - (theta).cos();
    let b1: f32 = (gamma * gamma - 1.0).sqrt() - gamma;
    let a0: f32 = 1.0 + b1;
    let a1: f32 = 0.0;

    FirstOrderFilter::new(FirstOrderFilterConfig { a0, a1, b1 })
}
