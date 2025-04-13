use egui::Ui;
use state::get_set;
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
) {
    let response = ui.add(
        egui::Slider::from_get_set(range, get_set(value.into(), |it| setter(it.into())))
            .text(label)
            .logarithmic(logarithmic),
    );

    if response.drag_stopped() || response.lost_focus() {
        on_release();
    }
}
