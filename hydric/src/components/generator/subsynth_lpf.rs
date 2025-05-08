use crate::view::View;
use crate::widget::knob;
use eframe::egui;
use egui::Ui;
use shared::model::SubSynthLpf;
use state::{Action, FloatField};

pub struct SubSynthLpfView<'a, F: Fn(Action), G: Fn()> {
    config: &'a SubSynthLpf,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SubSynthLpfView<'a, F, G> {
    pub fn new(config: &'a SubSynthLpf, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for SubSynthLpfView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config,
            dispatch,
            on_release,
        } = self;

        knob(
            ui,
            "Freq",
            config.fc,
            |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
            20.0..=20000.0,
            2000.0,
            || on_release(),
        );

        knob(
            ui,
            "Q",
            config.q,
            |it| dispatch(Action::SetFloat(FloatField::Q, it)),
            0.1..=100.0,
            1.0,
            || on_release(),
        );

        knob(
            ui,
            "Gain",
            config.gain,
            |it| dispatch(Action::SetFloat(FloatField::Gain, it)),
            -60.0..=60.0,
            0.0,
            || on_release(),
        );
    }
}
