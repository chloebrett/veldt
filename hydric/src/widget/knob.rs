use egui::{Color32, TextEdit, Ui};
pub use egui_fancy_knob::add_knob;
use egui_fancy_knob::{Knob, KnobStyle, LabelPosition};
use std::ops::RangeInclusive;

/// Struct to hold user input state for typable knob
pub struct TypableKnob {
    pub value: f32,
    pub text: String,
}

impl TypableKnob {
    pub fn new(initial: f32) -> Self {
        Self {
            value: initial,
            text: format!("{:.2}", initial),
        }
    }

    pub fn editable_knob<G>(
        &mut self,
        ui: &mut Ui,
        label: &str,
        range: RangeInclusive<f32>,
        setter: impl Fn(f32) + Copy,
        on_release: &G,
    )
    where
        G: Fn(),
    {
        let mut knob_value = self.value;

        ui.horizontal(|ui| {
            add_knob(
                ui,
                Knob::new(self.value, |v| {
                    knob_value = v;
                }, range.clone(), KnobStyle::Wiper)
                .with_size(20.0)
                .with_font_size(12.0)
                .with_stroke_width(2.0)
                .with_colors(Color32::GRAY, Color32::WHITE, Color32::WHITE, Color32::WHITE),
                on_release,
            );

            ui.label(format!("{label}:"));

            // Input textbox
            let text_response = ui.add(TextEdit::singleline(&mut self.text).desired_width(64.0));

            if text_response.changed() {
                if let Ok(mut v) = self.text.parse::<f32>() {
                    v = v.clamp(*range.start(), *range.end());
                    self.value = v;
                    self.text = format!("{:.2}", v);
                    setter(v);
                } else {
                    self.text = format!("{:.2}", self.value);
                }
            }

            // Update label value if not consistent if textbox is not focused
            if knob_value != self.value && !text_response.has_focus() {
                self.value = knob_value;
                self.text = format!("{:.2}", self.value);
                setter(self.value);
            }
        });
    }
}

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
