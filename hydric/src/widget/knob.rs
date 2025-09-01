use egui::{Color32, Response, TextEdit, Ui};
pub use egui_fancy_knob::add_knob;
use egui_fancy_knob::{Knob, KnobStyle};
use std::ops::RangeInclusive;

// Struct for knob with typable input to hold an updated version of the value at all times, consolidated from both the knob input and textbox input
pub struct TypableKnob {
    pub value: f32,
    pub text: String,
    desired_width: f32,
}

impl TypableKnob {
    pub fn new(initial: f32, desired_width: f32) -> Self {
        Self {
            value: initial,
            text: format!("{:.2}", initial),
            desired_width,
        }
    }

    pub fn show<F, G, H>(
        &mut self,
        ui: &mut Ui,
        knob: Knob<F>,
        label: &str,
        setter: G,
        range: RangeInclusive<f32>,
        on_release: &H,
    ) where
        F: Fn(f32),
        G: Fn(f32),
        H: Fn(),
    {
        let old_value = self.value;
        let step = 0.01;

        ui.horizontal(|ui| {
            add_knob(ui, knob, on_release);

            let value_changed_by_knob = self.value != old_value;

            if !label.is_empty() {
                ui.label(label);
            }

            let mut value_from_textbox = self.value;
            let dv = egui::DragValue::new(&mut value_from_textbox)
                .range(range.clone())
                .speed(step)
                .min_decimals(2)
                .max_decimals_opt(Some(2));

            let value_response = ui.add(dv);

            if value_response.changed() {
                self.value = value_from_textbox;
                self.text = format!("{:.2}", self.value);
                setter(self.value);
            }

            if value_changed_by_knob && !value_response.has_focus() {
                self.text = format!("{:.2}", self.value);
            }
        });
    }
}

pub fn styled_knob(
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
}

pub fn disabled_knob(value: f32, range: RangeInclusive<f32>) -> Knob<impl Fn(f32)> {
    styled_knob(value, |_| {}, range)
        .with_colors(
            Color32::DARK_GRAY,
            Color32::GRAY,
            Color32::GRAY,
            Color32::GRAY,
        )
        .enabled(false)
}

pub fn add_typable_knob<F, G, H>(
    ui: &mut Ui,
    knob: Knob<F>,
    label: &str,
    value: f32,
    setter: G,
    range: RangeInclusive<f32>,
    on_release: &H,
    desired_width: f32,
) where
    F: Fn(f32),
    G: Fn(f32),
    H: Fn(),
{
    TypableKnob::new(value, desired_width).show(ui, knob, label, setter, range, on_release);
}

pub fn add_disabled_knob<F, G>(
    ui: &mut Ui,
    knob: Knob<F>,
    label: &str,
    value: f32,
    range: RangeInclusive<f32>,
    on_release: &G,
    desired_width: f32,
) where
    F: Fn(f32),
    G: Fn(),
{
    TypableKnob::new(value, desired_width).show(ui, knob, label, |_| {}, range, on_release);
}
