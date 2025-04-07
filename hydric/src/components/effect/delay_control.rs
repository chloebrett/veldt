use crate::widget::{FloatRange, knob};
use egui::Ui;
use shared::model::DelayConfig;
use state::Action;

pub fn delay_control<F>(config: &DelayConfig, dispatch: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    knob(
        ui,
        "Delay ms",
        config.delay_ms,
        |it| dispatch(Action::SetDelayMs(it)),
        FloatRange(1.0, 1000.0),
    );
}
