use super::filter::{SecondOrderFilter, SecondOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn apf_second_order(config: &EqConfig) -> SecondOrderFilter {
    let bw = config.fc / config.q;
    let alpha_arg = ((bw * PI) / SAMPLE_RATE as f32).tan();
    let alpha = 1.0 + 1.0 / (alpha_arg - 1.0);
    let beta = -(2.0 * PI * config.fc / SAMPLE_RATE as f32).cos();
    let a0 = -alpha;
    let a1 = beta * (1.0 - alpha);
    let a2 = 1.0;
    let b1 = a1;
    let b2 = a0;
    SecondOrderFilter::new(SecondOrderFilterConfig {
        a0,
        a1,
        a2,
        b1,
        b2,
        c0: 1.0,
        d0: 0.0,
    })
}
