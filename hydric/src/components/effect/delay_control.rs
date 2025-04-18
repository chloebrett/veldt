use crate::widget::knob;
use egui::Ui;
use shared::model::DelayConfig;
use state::{Action, FloatField};

pub fn delay_control<F, G>(config: &DelayConfig, dispatch: F, on_release: G, ui: &mut Ui)
where
    F: Fn(Action),
    G: Fn(),
{
    knob(
        ui,
        "Delay ms",
        config.delay_ms,
        |it| dispatch(Action::SetFloat(FloatField::DelayMs, it)),
        1.0..=1000.0,
        &on_release,
    );
    knob(
        ui,
        "Feedback",
        config.feedback,
        |it| dispatch(Action::SetFloat(FloatField::Feedback, it)),
        0.0..=0.99,
        &on_release,
    );
}
