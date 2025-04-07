use ecolor::Color32;
use egui::Ui;
use egui_knob::{Knob, KnobStyle, LabelPosition};
use shared::types::KnobPosition;

pub struct FloatRange(pub f32, pub f32);

pub fn knob(
    ui: &mut Ui,
    label: &str,
    value: KnobPosition,
    setter: impl Fn(KnobPosition),
    range: FloatRange,
) {
    let FloatRange(min, max) = range;
    let mut temp = value.clamp(min, max);
    let knob = Knob::new(&mut temp, min, max, KnobStyle::Wiper)
        .with_size(20.0)
        .with_font_size(12.0)
        .with_stroke_width(2.0)
        .with_colors(Color32::GRAY, Color32::WHITE, Color32::WHITE)
        .with_label(label, LabelPosition::Right);

    ui.add(knob);

    if temp != value {
        setter(temp.clamp(min, max));
    }
}
