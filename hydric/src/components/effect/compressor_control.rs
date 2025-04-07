use crate::widget::{FloatRange, knob};
use egui::Ui;
use shared::model::CompressorConfig;
use state::Action;

pub fn compressor_control<F>(config: &CompressorConfig, dispatch: F, ui: &mut Ui)
where
    F: Fn(Action),
{
    knob(
        ui,
        "Threshold",
        config.threshold,
        |it| dispatch(Action::SetCompressorThreshold(it)),
        FloatRange(0.0, 1.0),
    );

    knob(
        ui,
        "Attack (ms)",
        config.attack_ms,
        |it| dispatch(Action::SetCompressorAttackMs(it)),
        FloatRange(0.0, 1000.0), // TODO: logarithmic
    );

    knob(
        ui,
        "Release (ms)",
        config.release_ms,
        |it| dispatch(Action::SetCompressorReleaseMs(it)),
        FloatRange(0.0, 1000.0), // TODO: logarithmic
    );

    knob(
        ui,
        "Ratio",
        config.ratio,
        |it| dispatch(Action::SetCompressorRatio(it)),
        FloatRange(1.0, 100.0),
    );
}
