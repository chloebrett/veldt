use egui::Ui;
use shared::model::DelayConfig;
use shared::types::Milliseconds;
use state::{Action, get_set};

pub fn delay_control<F>(config: DelayConfig, dispatch_effect: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    ui.label("Delay");

    ui.add(
        egui::Slider::from_get_set(
            1.0..=1000.0,
            get_set(config.delay_ms.into(), |it| {
                dispatch_effect(Action::SetDelayMs(it as Milliseconds))
            }),
        )
        .text("Delay ms"),
    );
}
