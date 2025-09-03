use crate::view::View;
use crate::widget::{add_typable_knob, styled_knob};
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
        let delay_knob = styled_knob(
            config.delay_ms,
            |it| dispatch(Action::SetFloat(FloatField::DelayMs, it)),
            1.0..=1000.0,
        )
        .with_neutral(100.0);
        let feedback_knob = styled_knob(
            config.feedback,
            |it| dispatch(Action::SetFloat(FloatField::Feedback, it)),
            0.0..=0.99,
        )
        .with_neutral(0.5);
        add_typable_knob(
            ui,
            delay_knob,
            "Delay (ms)",
            config.delay_ms,
            |it| dispatch(Action::SetFloat(FloatField::DelayMs, it)),
            1.0..=1000.0,
            &self.on_release,
        );
        add_typable_knob(
            ui,
            feedback_knob,
            "Feedback",
            config.feedback,
            |it| dispatch(Action::SetFloat(FloatField::Feedback, it)),
            0.0..=0.99,
            &self.on_release,
        );
    }
}
