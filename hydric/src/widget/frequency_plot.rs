use std::ops::RangeInclusive;

use egui::{
    Align2, CornerRadius, FontId, Pos2, Rangef, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
    lerp, pos2, remap_clamp, vec2,
};

enum FrequencySpec {
    Linear,
    Logarithmic {
        // The minimum frequency to display.
        min_frequency: f32,
        // X ticks when plot is made logarithmic.
        x_ticks: Vec<f32>,
    },
}

/// Display the frequency response of a signal on a 2D plot.
/// X-axis plots frequencies in Hz, y-axis plots response in dB.
/// Display can plot multiple frequency response lines.
/// X axis can be set to a logarithmic range.
/// Default y range is -60 to 10 dB.
/// Default x range is 0 to 22050 Hz.
pub struct FrequencyPlot<'a> {
    primary_freqs: &'a Vec<Pos2>,
    secondary_freqs: Vec<&'a Vec<Pos2>>,
    size: Vec2,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
    x_ticks: Vec<f32>,
    y_ticks: Vec<f32>,
    spec: FrequencySpec,
    text_size: f32,
    axis_line_stroke: Option<Stroke>,
}

#[allow(dead_code)]
impl<'a> FrequencyPlot<'a> {
    pub fn new(frequencies: &'a Vec<Pos2>) -> Self {
        let (x_min, x_max) = (0, 22050);
        let (y_min, y_max) = (10, -60);
        let x_ticks = (x_min..=x_max).step_by(2000).map(|it| it as f32).collect();
        let y_ticks = (y_max..=y_min).step_by(10).map(|it| it as f32).collect();
        Self {
            primary_freqs: frequencies,
            secondary_freqs: vec![],
            size: vec2(500.0, 250.0),
            x_range: x_min as f32..=x_max as f32,
            y_range: y_min as f32..=y_max as f32,
            x_ticks,
            y_ticks,
            spec: FrequencySpec::Linear,
            text_size: 12.0,
            axis_line_stroke: None,
        }
    }

    /// Add secondary frequency responses to the display.
    /// E.g. peak response.
    #[inline]
    pub fn add_secondary_frequencies(mut self, frequencies: &'a Vec<Pos2>) -> Self {
        self.secondary_freqs.push(frequencies);
        self
    }

    #[inline]
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub fn set_x_range(mut self, x_range: impl Into<RangeInclusive<f32>>) -> Self {
        self.x_range = x_range.into();
        self
    }

    #[inline]
    pub fn set_y_range(mut self, y_range: impl Into<RangeInclusive<f32>>) -> Self {
        self.y_range = y_range.into();
        self
    }

    /// Plot the frequency axis with a logarithmic scale.
    #[inline]
    pub fn logarithmic(mut self, logarithmic: bool) -> Self {
        self.spec = if logarithmic {
            FrequencySpec::Logarithmic {
                min_frequency: 10.0,
                x_ticks: vec![
                    10.0, 50.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 22050.0,
                ],
            }
        } else {
            FrequencySpec::Linear
        };
        self
    }

    /// For logarithmic plots.
    /// The minimum frequency that will be displayed. Default is 10.0 Hz.
    #[inline]
    pub fn min_frequency(mut self, min_frequency: f32) -> Self {
        self.spec = match self.spec {
            FrequencySpec::Logarithmic { x_ticks, .. } => FrequencySpec::Logarithmic {
                min_frequency,
                x_ticks,
            },
            FrequencySpec::Linear => {
                panic!("`min_frequency can only be set when Plot is logarithmic.")
            }
        };
        self
    }

    /// Set the stroke of the grid lines on the plot.
    #[inline]
    pub fn set_axis_line_stroke(mut self, stroke: Stroke) -> Self {
        self.axis_line_stroke = Some(stroke);
        self
    }

    #[inline]
    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size;
        self
    }

    fn axis_line_stroke(&self, ui: &Ui) -> Stroke {
        if let Some(stroke) = self.axis_line_stroke {
            stroke
        } else {
            Stroke {
                width: 0.15,
                color: ui.style().visuals.widgets.active.fg_stroke.color,
            }
        }
    }

    fn x_range(&self) -> RangeInclusive<f32> {
        self.x_range.clone()
    }

    fn y_range(&self) -> RangeInclusive<f32> {
        self.y_range.clone()
    }

    fn get_x_ticks(&self) -> &[f32] {
        match &self.spec {
            FrequencySpec::Linear => &self.x_ticks,
            FrequencySpec::Logarithmic { x_ticks, .. } => x_ticks,
        }
    }

    // Convert a `Pos2` in units (Hz, dB) to the raw (x, y) position on the plot.
    fn position_from_pos(
        &self,
        pos: Pos2,
        x_position_range: Rangef,
        y_position_range: Rangef,
    ) -> Pos2 {
        let normalised = normalised_from_pos(pos, self.x_range(), self.y_range(), &self.spec);
        lerp_pos(x_position_range, y_position_range, normalised)
    }

    fn line_ui(&self, ui: &Ui, rect: &Rect, line: &Vec<Pos2>, stroke: &Stroke) {
        let points: Vec<Pos2> = line
            .iter()
            .map(|&pos| self.position_from_pos(pos, rect.x_range(), rect.y_range()))
            .collect();
        ui.painter().line(points, *stroke);
    }

    fn plot_ui(&self, ui: &Ui, response: &Response) {
        let rect = response.rect;
        // Background shape.
        ui.painter().rect(
            rect,
            CornerRadius::ZERO,
            ui.style().visuals.extreme_bg_color,
            ui.style().visuals.window_stroke(),
            egui::StrokeKind::Middle,
        );
        let primary_stroke = ui.style().visuals.widgets.active.fg_stroke;
        self.line_ui(ui, &rect, self.primary_freqs, &primary_stroke);
        let secondary_stroke = ui.style().visuals.widgets.inactive.fg_stroke;
        for freq in &self.secondary_freqs {
            self.line_ui(ui, &rect, freq, &secondary_stroke);
        }
    }

    fn y_axis_ui(&self, ui: &Ui, response: &Response) {
        let rect = response.rect;
        let x_range = self.x_range();
        let font_id = FontId::proportional(self.text_size);
        let text_color = ui.style().visuals.text_color();
        for y in &self.y_ticks {
            let pos = pos2(*x_range.start(), *y);
            let position = self.position_from_pos(pos, rect.x_range(), rect.y_range());
            // Add padding so values do not sit directly on axis.
            let position = position - vec2(self.text_size * 0.5, 0.0);
            // Y tick label.
            ui.painter().text(
                position,
                Align2::RIGHT_CENTER,
                format!("{y}"),
                font_id.clone(),
                text_color,
            );
            // Y tick axis line
            ui.painter().line(
                vec![
                    pos2(rect.left(), position.y),
                    pos2(rect.right(), position.y),
                ],
                self.axis_line_stroke(ui),
            );
        }
    }

    fn x_axis_ui(&self, ui: &Ui, response: &Response) {
        let rect = response.rect;
        let y_range = self.y_range();
        let font_id = FontId::proportional(self.text_size);
        let text_color = ui.style().visuals.text_color();
        let x_ticks = self.get_x_ticks();
        for x in x_ticks {
            let pos = pos2(*x, *y_range.end());
            let position = self.position_from_pos(pos, rect.x_range(), rect.y_range());
            // Add padding so values do not sit directly on axis.
            let position = position + vec2(0.0, self.text_size * 0.5);
            // X tick label.
            ui.painter().text(
                position,
                Align2::CENTER_TOP,
                format!("{x}"),
                font_id.clone(),
                text_color,
            );
            // X tick axis line
            ui.painter().line(
                vec![
                    pos2(position.x, rect.bottom()),
                    pos2(position.x, rect.top()),
                ],
                self.axis_line_stroke(ui),
            );
        }
    }

    fn add_contents(&self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(self.size, Sense::focusable_noninteractive());
        // Shrink size to account for x and y axis labels.
        let shrink_amount = self.text_size;
        let rect = response
            .rect
            .shrink2(vec2(shrink_amount, shrink_amount))
            .translate(vec2(shrink_amount, -shrink_amount));
        let id = response.id.with("plot");
        let response = ui.interact(rect, id, Sense::focusable_noninteractive());
        self.plot_ui(ui, &response);
        self.y_axis_ui(ui, &response);
        self.x_axis_ui(ui, &response);
        response
    }
}

impl Widget for FrequencyPlot<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.add_contents(ui)
    }
}

// Helper functions to convert Pos2 to and from the [0, 1] x and y range.
// Also accounts for logarithmic x axis.

fn normalised_from_pos(
    pos: Pos2,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
    spec: &FrequencySpec,
) -> Pos2 {
    match spec {
        FrequencySpec::Logarithmic { min_frequency, .. } => {
            let (x_min, x_max) = (*x_range.start(), *x_range.end());
            let log_pos = pos2(pos.x.max(*min_frequency).log10(), pos.y);
            remap_clamp_pos(
                log_pos,
                x_min.max(*min_frequency).log10()..=x_max.log10(),
                y_range,
                0.0..=1.0,
                0.0..=1.0,
            )
        }
        FrequencySpec::Linear => remap_clamp_pos(pos, x_range, y_range, 0.0..=1.0, 0.0..=1.0),
    }
}

fn remap_clamp_pos(
    pos: Pos2,
    x_from: impl Into<RangeInclusive<f32>>,
    y_from: impl Into<RangeInclusive<f32>>,
    x_to: impl Into<RangeInclusive<f32>>,
    y_to: impl Into<RangeInclusive<f32>>,
) -> Pos2 {
    pos2(
        remap_clamp(pos.x, x_from, x_to),
        remap_clamp(pos.y, y_from, y_to),
    )
}

fn lerp_pos(
    x_range: impl Into<RangeInclusive<f32>>,
    y_range: impl Into<RangeInclusive<f32>>,
    pos: Pos2,
) -> Pos2 {
    pos2(lerp(x_range, pos.x), lerp(y_range, pos.y))
}
