use super::filter::FirstOrderFilter;
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn apf_first_order(config: &EqConfig) -> FirstOrderFilter {
    let alpha_arg = ((config.fc * PI) / SAMPLE_RATE as f32).tan();
    let alpha = (alpha_arg - 1.0) / (alpha_arg + 1.0);
    let a0 = -alpha;
    let a1 = 1.0;
    let b1 = -alpha;
    FirstOrderFilter { a0, a1, b1 }
}
