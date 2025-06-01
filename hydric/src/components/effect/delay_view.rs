use crate::view::View;
use crate::widget::{add_knob, styled_knob};
use egui::Ui;
use shared::model::DelayConfig;
use state::{Action, FloatField};

pub struct DelayView<'a, F: Fn(Action), G: Fn()> {
    config: &'a DelayConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> DelayView<'a, F, G> {
    pub fn new(config: &'a DelayConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for DelayView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config, dispatch, ..
        } = self;

        add_knob(
            ui,
            styled_knob(
                "Delay ms",
                config.delay_ms,
                |it| dispatch(Action::SetFloat(FloatField::DelayMs, it)),
                1.0..=1000.0,
            )
            .with_neutral(100.0),
            &self.on_release,
        );
        add_knob(
            ui,
            styled_knob(
                "Feedback",
                config.feedback,
                |it| dispatch(Action::SetFloat(FloatField::Feedback, it)),
                0.0..=0.99,
            )
            .with_neutral(0.5),
            &self.on_release,
        );
    }
}
