use super::filter::{Filter, FilterConfig};
use crate::consts::SAMPLE_RATE;
use shared::model::EqConfig;
use std::f32::consts::PI;

pub fn apf_first_order(config: &EqConfig) -> Filter {
    let alpha_arg = ((PI * config.fc) / SAMPLE_RATE as f32).tan();
    let alpha = 1.0 + 1.0 / (alpha_arg - 1.0);
    let a0 = -alpha;
    let a1 = 1.0;
    let b1 = -alpha;

    Filter::new_wet(FilterConfig::default().a0(a0).a1(a1).b1(b1))
}
