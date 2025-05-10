use crate::view::View;
use crate::widget::knob;
use egui::Ui;
use shared::model::CompressorConfig;
use state::{Action, FloatField};

pub struct CompressorView<'a, F: Fn(Action), G: Fn()> {
    config: &'a CompressorConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> CompressorView<'a, F, G> {
    pub fn new(config: &'a CompressorConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for CompressorView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config, dispatch, ..
        } = self;

        knob(
            ui,
            "Threshold",
            config.threshold,
            |it| dispatch(Action::SetFloat(FloatField::Threshold, it)),
            -50.0..=0.0,
            /* neutral= */ -10.0,
            &self.on_release,
        );

        knob(
            ui,
            "Attack (ms)",
            config.attack_ms,
            |it| dispatch(Action::SetFloat(FloatField::AttackMs, it)),
            0.0..=1000.0,
            /* neutral= */ 100.0,
            &self.on_release,
        );

        knob(
            ui,
            "Release (ms)",
            config.release_ms,
            |it| dispatch(Action::SetFloat(FloatField::ReleaseMs, it)),
            0.0..=1000.0,
            /* neutral= */ 100.0,
            &self.on_release,
        );

        knob(
            ui,
            "Ratio",
            config.ratio,
            |it| dispatch(Action::SetFloat(FloatField::Ratio, it)),
            1.0..=100.0, // TODO: logarithmic
            /* neutral= */ 3.0,
            &self.on_release,
        );
    }
}
