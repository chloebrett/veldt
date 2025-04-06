use egui::Ui;
use shared::model::CompressorConfig;
use shared::types::KnobPosition;
use shared::types::Milliseconds;
use shared::types::Volume;
use state::{Action, get_set};

pub fn compressor_control<F>(config: &CompressorConfig, dispatch: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    ui.add(
        egui::Slider::from_get_set(
            0.0..=1.0,
            get_set(config.threshold.into(), |it| {
                dispatch(Action::SetCompressorThreshold(it as Volume));
            }),
        )
        .text("Threshold"),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1000.0,
            get_set(config.attack_ms.into(), |it| {
                dispatch(Action::SetCompressorAttackMs(it as Milliseconds));
            }),
        )
        .text("Attack"),
    );

    ui.add(
        egui::Slider::from_get_set(
            0.0..=1000.0,
            get_set(config.release_ms.into(), |it| {
                dispatch(Action::SetCompressorReleaseMs(it as Milliseconds));
            }),
        )
        .text("Release"),
    );

    ui.add(
        egui::Slider::from_get_set(
            1.0..=f64::INFINITY,
            get_set(config.ratio.into(), |it| {
                dispatch(Action::SetCompressorRatio(it as KnobPosition));
            }),
        )
        .text("Ratio")
        .logarithmic(true)
        .largest_finite(10000.0),
    );
}
