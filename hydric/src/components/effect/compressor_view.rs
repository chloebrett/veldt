use crate::widget::{add_knob, styled_knob};
use crate::{transform::Transform, view::View};
use egui::Color32;
use egui::{
    Frame, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget, emath::RectTransform, pos2, vec2,
};
use shared::model::CompressorConfig;
use state::{Action, FloatField};

pub struct CompressorView<'a, F: Fn(Action), G: Fn()> {
    config: &'a CompressorConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> CompressorView<'a, F, G> {
    pub fn new(config: &'a CompressorConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for CompressorView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config, dispatch, ..
        } = self;
        ui.horizontal(|ui| {
            ui.add(CompressorDisplay::new(config));
            ui.vertical(|ui| {
                add_knob(
                    ui,
                    styled_knob(
                        "Threshold",
                        config.threshold,
                        |it| dispatch(Action::SetFloat(FloatField::Threshold, it)),
                        -60.0..=0.0,
                    )
                    .with_neutral(-10.0),
                    &self.on_release,
                );
                add_knob(
                    ui,
                    styled_knob(
                        "Ratio",
                        config.ratio,
                        |it| dispatch(Action::SetFloat(FloatField::Ratio, it)),
                        1.0..=100.0,
                    )
                    .with_neutral(3.0), // TODO: logarithmic
                    &self.on_release,
                );
            });
            ui.vertical(|ui| {
                add_knob(
                    ui,
                    styled_knob(
                        "Attack (ms)",
                        config.attack_ms,
                        |it| dispatch(Action::SetFloat(FloatField::AttackMs, it)),
                        0.0..=1000.0,
                    )
                    .with_neutral(100.0),
                    &self.on_release,
                );
                add_knob(
                    ui,
                    styled_knob(
                        "Release (ms)",
                        config.release_ms,
                        |it| dispatch(Action::SetFloat(FloatField::ReleaseMs, it)),
                        0.0..=1000.0,
                    )
                    .with_neutral(100.0),
                    &self.on_release,
                );
            });
        });
    }
}

struct CompressorDisplay<'a> {
    size: Vec2,
    config: &'a CompressorConfig,
}

impl<'a> CompressorDisplay<'a> {
    pub fn new(config: &'a CompressorConfig) -> Self {
        Self {
            size: vec2(50.0, 50.0),
            config,
        }
    }
}

/// A widget to display the "curve" of the compressor.
impl Widget for CompressorDisplay<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let CompressorDisplay { size, config } = self;
        // TODO: Add gain and knee when implemented.
        // TODO: Add live level to show compression.
        let CompressorConfig {
            threshold, ratio, ..
        } = *config;
        let max_db = 0.0;
        let min_db = -60.0;
        Frame::canvas(ui.style())
            .show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(size, Sense::click());
                let range = Rect::from_min_max(pos2(min_db, max_db), pos2(max_db, min_db));
                let to_screen = RectTransform::from_to(range, response.rect);
                // Level pre compression under threshold.
                let min = pos2(min_db, min_db);
                let knee = pos2(threshold, threshold);
                // Level post compression over threshold.
                let max = pos2(max_db, threshold + (max_db - threshold) / ratio);
                let line = Shape::line(vec![min, knee, max], Stroke::new(1.0, Color32::WHITE));
                painter.add(line.transform(to_screen));
                response
            })
            .response
    }
}
