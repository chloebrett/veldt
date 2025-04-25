use crate::view::View;
use crate::widget::knob;
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
        DelayView {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for DelayView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let DelayView {
            config, dispatch, ..
        } = self;

        knob(
            ui,
            "Delay ms",
            config.delay_ms,
            |it| dispatch(Action::SetFloat(FloatField::DelayMs, it)),
            1.0..=1000.0,
            /* neutral= */ 100.0,
            &self.on_release,
        );
        knob(
            ui,
            "Feedback",
            config.feedback,
            |it| dispatch(Action::SetFloat(FloatField::Feedback, it)),
            0.0..=0.99,
            /* neutral= */ 0.5,
            &self.on_release,
        );
    }
}
