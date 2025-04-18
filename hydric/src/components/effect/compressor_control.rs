use crate::widget::knob;
use egui::Ui;
use shared::model::CompressorConfig;
use state::{Action, FloatField};

pub fn compressor_control<F, G>(config: &CompressorConfig, dispatch: F, on_release: G, ui: &mut Ui)
where
    F: Fn(Action),
    G: Fn(),
{
    knob(
        ui,
        "Threshold",
        config.threshold,
        |it| dispatch(Action::SetFloat(FloatField::Threshold, it)),
        0.0..=1.0,
        &on_release,
    );

    knob(
        ui,
        "Attack (ms)",
        config.attack_ms,
        |it| dispatch(Action::SetFloat(FloatField::AttackMs, it)),
        0.0..=1000.0, // TODO: logarithmic
        &on_release,
    );

    knob(
        ui,
        "Release (ms)",
        config.release_ms,
        |it| dispatch(Action::SetFloat(FloatField::ReleaseMs, it)),
        0.0..=1000.0, // TODO: logarithmic
        &on_release,
    );

    knob(
        ui,
        "Ratio",
        config.ratio,
        |it| dispatch(Action::SetFloat(FloatField::Ratio, it)),
        1.0..=100.0,
        &on_release,
    );
}
