use egui::Color32;
pub use egui_fancy_knob::add_knob;
use egui_fancy_knob::{Knob, KnobStyle, LabelPosition};
use std::ops::RangeInclusive;

pub fn styled_knob(
    label: &str,
    value: f32,
    setter: impl Fn(f32),
    range: RangeInclusive<f32>,
) -> Knob<impl Fn(f32)> {
    Knob::new(value, setter, range, KnobStyle::Wiper)
        .with_size(20.0)
        .with_font_size(12.0)
        .with_stroke_width(2.0)
        .with_colors(
            Color32::GRAY,
            Color32::WHITE,
            Color32::WHITE,
            Color32::WHITE,
        )
        .with_label(label, LabelPosition::Right)
        .with_label_offset(4.0)
}

pub fn disabled_knob(label: &str, value: f32, range: RangeInclusive<f32>) -> Knob<impl Fn(f32)> {
    styled_knob(label, value, |_| {}, range)
        .with_colors(
            Color32::DARK_GRAY,
            Color32::GRAY,
            Color32::GRAY,
            Color32::GRAY,
        )
        .enabled(false)
}
