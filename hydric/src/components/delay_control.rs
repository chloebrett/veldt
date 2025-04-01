use egui::Ui;
use shared::model::{DelayConfig, EffectMeta};
use shared::types::{KnobPosition, Milliseconds};
use state::{Action, get_set};

pub fn delay_control<F>(config: DelayConfig, meta: EffectMeta, dispatch_effect: F, ui: &mut Ui)
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
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(meta.wet.into(), |it| {
                dispatch_effect(Action::SetEffectWet(it as KnobPosition))
            }),
        )
        .text("Delay wet"),
    );
}
