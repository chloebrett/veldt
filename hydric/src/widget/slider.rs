use super::get_set;
use egui::Ui;
use std::ops::RangeInclusive;

pub fn slider<T: PartialEq + Clone + Into<f64> + From<f64>, F: Fn(T), G: Fn()>(
    ui: &mut Ui,
    label: &str,
    value: T,
    setter: F,
    range: RangeInclusive<f64>,
    on_release: G,
) {
    slider_internal(
        ui, label, value, setter, range, on_release, /* logarithmic= */ false,
        /* fixed_decimals= */ None,
    );
}

pub fn log_slider<T: PartialEq + Clone + Into<f64> + From<f64>, F: Fn(T), G: Fn()>(
    ui: &mut Ui,
    label: &str,
    value: T,
    setter: F,
    range: RangeInclusive<f64>,
    on_release: G,
) {
    slider_internal(
        ui, label, value, setter, range, on_release, /* logarithmic= */ true,
        /* fixed_decimals= */ None,
    );
}

pub fn int_slider<T: PartialEq + Clone + Into<f64> + From<f64>, F: Fn(T), G: Fn()>(
    ui: &mut Ui,
    label: &str,
    value: T,
    setter: F,
    range: RangeInclusive<i32>,
    on_release: G,
) {
    slider_internal(
        ui,
        label,
        value,
        setter,
        (*range.start() as f32).into()..=(*range.end() as f32).into(),
        on_release,
        /* logarithmic= */ true,
        /* fixed_decimals= */ Some(0),
    );
}

fn slider_internal<T: PartialEq + Clone + Into<f64> + From<f64>, F: Fn(T), G: Fn()>(
    ui: &mut Ui,
    label: &str,
    value: T,
    setter: F,
    range: RangeInclusive<f64>,
    on_release: G,
    logarithmic: bool,
    fixed_decimals: Option<usize>,
) {
    let mut slider =
        egui::Slider::from_get_set(range, get_set(value.into(), |it| setter(it.into())))
            .text(label)
            .logarithmic(logarithmic);

    if let Some(fixed_decimals) = fixed_decimals {
        slider = slider.fixed_decimals(fixed_decimals);
    }

    let response = ui.add(slider);

    if response.drag_stopped() || response.lost_focus() {
        on_release();
    }
}
