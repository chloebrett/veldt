use egui::{Align2, Color32, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

const KNOB_FINE_DRAG_RATIO: f32 = 0.2;

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

pub fn add_knob<F: Fn()>(ui: &mut Ui, knob: Knob<impl FnMut(f32)>, on_release: F) {
    let response = ui.add(knob);

    if response.drag_stopped() || response.lost_focus() {
        on_release();
    }
}

/// Position of the label relative to the knob
pub enum LabelPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// Visual style of the knob indicator
pub enum KnobStyle {
    /// A line extending from the center to the edge
    Wiper,
    /// A dot on the edge of the knob
    Dot,
}

/// A circular knob widget for egui that can be dragged to change a value
///
/// # Example
/// ```
/// let mut value = 0.5;
/// Knob::new(&mut value, 0.0, 1.0, KnobStyle::Wiper)
///     .with_size(50.0)
///     .with_label("Volume", LabelPosition::Bottom)
///     .with_step(0.1);
/// ```
pub struct Knob<F: FnMut(f32)> {
    value: f32,
    set_value: F,
    range: RangeInclusive<f32>,
    size: f32,
    font_size: f32,
    stroke_width: f32,
    knob_color: Color32,
    knob_dragging_color: Color32,
    line_color: Color32,
    text_color: Color32,
    label: Option<String>,
    label_position: LabelPosition,
    style: KnobStyle,
    label_offset: f32,
    label_format: Box<dyn FnMut(f32) -> String>,
    step: Option<f32>,
    neutral: Option<f32>,
    enabled: bool,
}

impl<F: FnMut(f32)> Knob<F> {
    /// Creates a new knob widget
    ///
    /// # Arguments
    /// * `value` - Mutable reference to the value controlled by the knob
    /// * `min` - Minimum value
    /// * `max` - Maximum value
    /// * `style` - Visual style of the knob indicator
    pub fn new(value: f32, set_value: F, range: RangeInclusive<f32>, style: KnobStyle) -> Self {
        Self {
            value: value.clamp(*range.start(), *range.end()),
            set_value,
            range,
            size: 40.0,
            font_size: 12.0,
            stroke_width: 2.0,
            knob_color: Color32::GRAY,
            knob_dragging_color: Color32::WHITE,
            line_color: Color32::GRAY,
            text_color: Color32::WHITE,
            label: None,
            label_position: LabelPosition::Bottom,
            style,
            label_offset: 1.0,
            label_format: Box::new(|v| format!("{:.2}", v)),
            step: None,
            neutral: None,
            enabled: true,
        }
    }

    /// Sets the size of the knob
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the font size for the label
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Sets the stroke width for the knob's outline and indicator
    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Sets the colors for different parts of the knob
    ///
    /// # Arguments
    /// * `knob_color` - Color of the knob's outline
    /// * `line_color` - Color of the indicator
    /// * `text_color` - Color of the label text
    pub fn with_colors(
        mut self,
        knob_color: Color32,
        knob_dragging_color: Color32,
        line_color: Color32,
        text_color: Color32,
    ) -> Self {
        self.knob_color = knob_color;
        self.knob_dragging_color = knob_dragging_color;
        self.line_color = line_color;
        self.text_color = text_color;
        self
    }

    /// Adds a label to the knob
    ///
    /// # Arguments
    /// * `label` - Text to display
    /// * `position` - Position of the label relative to the knob
    pub fn with_label(mut self, label: impl Into<String>, position: LabelPosition) -> Self {
        self.label = Some(label.into());
        self.label_position = position;
        self
    }

    /// Sets the spacing between the knob and its label
    pub fn with_label_offset(mut self, offset: f32) -> Self {
        self.label_offset = offset;
        self
    }

    /// Sets a custom format function for displaying the value
    ///
    /// # Example
    /// ```
    /// # let mut value = 0.5;
    /// Knob::new(&mut value, 0.0, 1.0, KnobStyle::Wiper)
    ///     .with_label_format(|v| format!("{:.1}%", v * 100.0));
    /// ```
    pub fn with_label_format(mut self, format: impl FnMut(f32) -> String + 'static) -> Self {
        self.label_format = Box::new(format);
        self
    }

    /// Sets the step size for value changes.
    ///
    /// When set, the value will snap to discrete steps as the knob is dragged.
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets the neutral value.
    ///
    /// When the knob is double clicked, it will reset to the neutral value.
    pub fn with_neutral(mut self, neutral: f32) -> Self {
        self.neutral = Some(neutral);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<F: FnMut(f32)> Widget for Knob<F> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let knob_size = Vec2::splat(self.size);
        let min = *self.range.start();
        let max = *self.range.end();

        let label_size = if let Some(label) = &self.label {
            let font_id = egui::FontId::proportional(self.font_size);
            let max_text = format!("{}: {}", label, (self.label_format)(max));
            ui.painter()
                .layout(max_text, font_id, Color32::WHITE, f32::INFINITY)
                .size()
        } else {
            Vec2::ZERO
        };

        let label_padding = 2.0;
        let vertical_margin = 4.0;

        ui.add_space(vertical_margin);

        let adjusted_size = match self.label_position {
            LabelPosition::Top | LabelPosition::Bottom => Vec2::new(
                knob_size.x.max(label_size.x + label_padding * 2.0),
                knob_size.y + label_size.y + label_padding * 2.0 + self.label_offset,
            ),
            LabelPosition::Left | LabelPosition::Right => Vec2::new(
                knob_size.x + label_size.x + label_padding * 2.0 + self.label_offset,
                knob_size.y.max(label_size.y + label_padding * 2.0),
            ),
        };

        let (rect, mut response) = ui.allocate_exact_size(adjusted_size, Sense::click_and_drag());

        if self.enabled {
            // Double click to return to neutral state.
            if response.double_clicked() {
                if let Some(neutral) = self.neutral {
                    if neutral != self.value {
                        (self.set_value)(neutral);
                        response.mark_changed();
                    }
                }
            } else if response.dragged() {
                let mut delta = response.drag_delta().y;

                // Hold ctrl, alt or shift to move finely.
                ui.input(|input| {
                    if input.modifiers.ctrl || input.modifiers.shift || input.modifiers.alt {
                        delta *= KNOB_FINE_DRAG_RATIO;
                    }
                });

                let step = self.step.unwrap_or((max - min) * 0.005);
                let mut new_value = (self.value - delta * step).clamp(min, max);

                if let Some(step) = self.step {
                    let steps = ((new_value - min) / step).round();
                    new_value = (min + steps * step).clamp(min, max)
                }

                if new_value != self.value {
                    (self.set_value)(new_value);
                    response.mark_changed();
                }
            }
        }

        let is_dragging = response.dragged() && self.enabled;
        let painter = ui.painter();
        let knob_rect = match self.label_position {
            LabelPosition::Left => {
                Rect::from_min_size(rect.right_top() + Vec2::new(-knob_size.x, 0.0), knob_size)
            }
            LabelPosition::Right => Rect::from_min_size(rect.left_top(), knob_size),
            LabelPosition::Top => Rect::from_min_size(
                rect.left_bottom() + Vec2::new((rect.width() - knob_size.x) / 2.0, -knob_size.y),
                knob_size,
            ),
            LabelPosition::Bottom => Rect::from_min_size(
                rect.left_top() + Vec2::new((rect.width() - knob_size.x) / 2.0, 0.0),
                knob_size,
            ),
        };

        let center = knob_rect.center();
        let radius = if is_dragging {
            knob_size.x * 0.55
        } else {
            knob_size.x * 0.5
        };

        // The range of motion of the knob. 1.0 means a full rotation.
        let range = 0.85;

        // 0.0 points right. 0.25 points down.
        let down = 0.25;

        // The necessary offset from pointing down, in order for motion to be symmetrical.
        let offset = (1.0 - range) * 0.5;

        let start_angle = down + offset;

        let angle = TAU * ((self.value - min) / (max - min) * range + start_angle);

        let knob_color = if is_dragging {
            self.knob_dragging_color
        } else {
            self.knob_color
        };
        painter.circle_stroke(center, radius, Stroke::new(self.stroke_width, knob_color));

        match self.style {
            KnobStyle::Wiper => {
                let pointer = center + Vec2::angled(angle) * (radius * 0.7);
                painter.line_segment(
                    [center, pointer],
                    Stroke::new(self.stroke_width * 1.5, self.line_color),
                );
            }
            KnobStyle::Dot => {
                let dot_pos = center + Vec2::angled(angle) * (radius * 0.7);
                painter.circle_filled(dot_pos, self.stroke_width * 1.5, self.line_color);
            }
        }

        if let Some(label) = self.label {
            let value_string = (self.label_format)(self.value);
            let label_text = if label.is_empty() {
                // If the label is empty, format only the value string
                value_string.to_string()
            } else {
                // If the label is not empty, format with the label, colon, and value string
                format!("{}: {}", label, value_string)
            };
            let font_id = egui::FontId::proportional(self.font_size);

            let (label_pos, alignment) = match self.label_position {
                LabelPosition::Top => (
                    Vec2::new(
                        rect.center().x,
                        rect.min.y - self.label_offset + label_padding,
                    ),
                    Align2::CENTER_TOP,
                ),
                LabelPosition::Bottom => (
                    Vec2::new(rect.center().x, rect.max.y + self.label_offset),
                    Align2::CENTER_BOTTOM,
                ),
                LabelPosition::Left => (
                    Vec2::new(rect.min.x - self.label_offset, rect.center().y),
                    // Might be wrong!
                    Align2::LEFT_CENTER,
                ),
                LabelPosition::Right => (
                    Vec2::new(rect.max.x - label_size.x, rect.center().y),
                    // Fixed this - it was right_center before which caused alignment issues.
                    Align2::LEFT_CENTER,
                ),
            };

            ui.painter().text(
                label_pos.to_pos2(),
                alignment,
                label_text,
                font_id,
                self.text_color,
            );
        }

        if cfg!(feature = "extra_debug") {
            // Draw the bounding rect
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, Color32::RED),
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                knob_rect,
                0.0,
                Stroke::new(1.0, Color32::GREEN),
                egui::StrokeKind::Inside,
            );
        }

        ui.add_space(vertical_margin);

        response
    }
}
