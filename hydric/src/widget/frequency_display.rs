use std::ops::RangeInclusive;

use egui::{
    Align2, CornerRadius, FontId, Pos2, Rangef, Rect, Response, Sense, Stroke, Ui, Vec2,
    Widget, lerp, pos2, remap, remap_clamp, vec2,
};

struct FrequencySpec {
    logarithmic: bool,
    // For logarithmic frequency displays.
    // The minimum frequency to display.
    min_frequency: f32,
}

pub struct FrequencyDisplay<'a> {
    primary_freqs: &'a Vec<Pos2>,
    secondary_freqs: Vec<&'a Vec<Pos2>>,
    size: Vec2,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
    spec: FrequencySpec,
    text_size: f32,
    axis_line_stroke: Option<Stroke>,
}

#[allow(dead_code)]
impl<'a> FrequencyDisplay<'a> {
    pub fn new(frequencies: &'a Vec<Pos2>) -> Self {
        Self {
            primary_freqs: frequencies,
            secondary_freqs: vec![],
            size: vec2(500.0, 250.0),
            x_range: 0.0..=21500.0,
            y_range: 10.0..=-60.0,
            spec: FrequencySpec {
                logarithmic: false,
                min_frequency: 10.0,
            },
            text_size: 12.0,
            axis_line_stroke: None,
        }
    }

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

    #[inline]
    pub fn logarithmic(mut self, logarithmic: bool) -> Self {
        self.spec.logarithmic = logarithmic;
        self
    }

    #[inline]
    pub fn min_frequency(mut self, min_frequency: f32) -> Self {
        self.spec.min_frequency = min_frequency;
        self
    }

    #[inline]
    pub fn set_axis_line_stroke(mut self, stroke: Stroke) -> Self {
        self.axis_line_stroke = Some(stroke);
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
        let range = self.y_range();
        let (min, max) = (*range.start() as i32, *range.end() as i32);
        let font_id = FontId::proportional(self.text_size);
        let text_color = ui.style().visuals.text_color();
        for y in (max..=min).step_by(10) {
            let y = y as f32;
            let height = remap_clamp(y, self.y_range(), rect.y_range());
            let pos = pos2(rect.left(), height);
            ui.painter().text(
                pos,
                Align2::RIGHT_CENTER,
                format!("{y}"),
                font_id.clone(),
                text_color,
            );
            ui.painter().line(
                vec![pos2(rect.left(), height), pos2(rect.right(), height)],
                self.axis_line_stroke(ui),
            );
        }
    }

    fn x_axis_ui(&self, ui: &Ui, response: &Response) {
        let rect = response.rect;
        let font_id = FontId::proportional(self.text_size);
        let text_color = ui.style().visuals.text_color();
        let range = self.x_range();
        if !self.spec.logarithmic {
            let (min, max) = (*range.start() as i32, *range.end() as i32);
            for x in (min..=max).step_by(2000) {
                let x = x as f32;
                let x_pos = remap_clamp(x, self.x_range(), rect.x_range());
                let pos = pos2(x_pos, rect.bottom());
                ui.painter().text(
                    pos,
                    Align2::CENTER_TOP,
                    format!("{x}"),
                    font_id.clone(),
                    text_color,
                );
                ui.painter().line(
                    vec![pos2(x_pos, rect.bottom()), pos2(x_pos, rect.top())],
                    self.axis_line_stroke(ui),
                );
            }
        } else {
            let min = self.spec.min_frequency.log10();
            let max = range.end().log10();
            let size = max - min;
            let step = size * 0.1;
            for x in 0..=10 {
                let x_pos = remap(x as f32, 0.0..=10.0, response.rect.x_range());
                let pos = pos2(x_pos, rect.bottom());
                ui.painter().text(
                    pos,
                    Align2::CENTER_TOP,
                    format!(
                        "{:.0?}",
                        10f32.powf(x as f32 * step) * self.spec.min_frequency
                    ),
                    font_id.clone(),
                    text_color,
                );
                let points = vec![
                    pos2(x_pos, response.rect.bottom()),
                    pos2(x_pos, response.rect.top()),
                ];
                ui.painter().line(points, self.axis_line_stroke(ui));
            }
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

impl Widget for FrequencyDisplay<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.add_contents(ui)
    }
}

fn normalised_from_pos(
    pos: Pos2,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
    spec: &FrequencySpec,
) -> Pos2 {
    if spec.logarithmic {
        let min_frequency = spec.min_frequency;
        let (x_min, x_max) = (*x_range.start(), *x_range.end());
        let log_pos = pos2(pos.x.max(min_frequency).log10(), pos.y);
        remap_clamp_pos(
            log_pos,
            x_min.max(min_frequency).log10()..=x_max.log10(),
            y_range,
            0.0..=1.0,
            0.0..=1.0,
        )
    } else {
        remap_clamp_pos(pos, x_range, y_range, 0.0..=1.0, 0.0..=1.0)
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
