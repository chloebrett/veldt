use super::filter::{FirstOrderFilter, FirstOrderFilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn apf_first_order(config: &EqConfig) -> FirstOrderFilter {
    let alpha_arg = ((PI * config.fc) / SAMPLE_RATE as f32).tan();
    let alpha = 1.0 + 1.0 / (alpha_arg - 1.0);
    let a0 = -alpha;
    let a1 = 1.0;
    let b1 = -alpha;

    FirstOrderFilter::new(FirstOrderFilterConfig { a0, a1, b1 })
}
