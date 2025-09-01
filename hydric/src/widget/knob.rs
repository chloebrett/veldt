use egui::{Color32, TextEdit, Ui};
pub use egui_fancy_knob::add_knob;
use egui_fancy_knob::{Knob, KnobStyle};
use std::ops::RangeInclusive;

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

    /// knob needs to be created fully outside
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
        let mut knob_value = self.value;

        ui.horizontal(|ui| {
            // Pass the fully configured knob in
            add_knob(ui, knob, on_release);

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

            if knob_value != self.value && !text_response.has_focus() {
                self.value = knob_value;
                self.text = format!("{:.2}", self.value);
                setter(self.value);
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
) where
    F: Fn(f32),
    G: Fn(f32),
    H: Fn(),
{
    TypableKnob::new(value).show(ui, knob, label, setter, range, on_release);
}

pub fn add_disabled_knob<F, G> (
    ui: &mut Ui,
    knob: Knob<F>,
    label: &str,
    value: f32,
    range: RangeInclusive<f32>,
    on_release: &G,
) where
    F: Fn(f32),
    G: Fn(),
{
    TypableKnob::new(value).show(ui, knob, label, |_| {}, range, on_release);
}