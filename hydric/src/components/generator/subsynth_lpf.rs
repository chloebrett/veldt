use crate::view::View;
use crate::widget::{knob, outer_frame};
use eframe::egui;
use egui::Ui;
use shared::model::EqConfig;
use state::{Action, FloatField};

pub struct SubSynthLpfView<'a, F: Fn(Action), G: Fn()> {
    config: &'a EqConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SubSynthLpfView<'a, F, G> {
    pub fn new(config: &'a EqConfig, dispatch: F, on_release: G) -> Self {
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

        let on_release = || on_release();
        outer_frame().inner_margin(10.0).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Low Pass Filter");
                ui.add_space(4.0);
            });
            knob(
                ui,
                "Freq",
                config.fc,
                |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                20.0..=20000.0,
                2000.0,
                on_release,
            );

            knob(
                ui,
                "Q",
                config.q,
                |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                0.1..=100.0,
                1.0,
                on_release,
            );
        });
    }
}
