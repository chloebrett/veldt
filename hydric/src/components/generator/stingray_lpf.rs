use crate::view::View;
use crate::widget::{add_knob, outer_frame, styled_knob, checkbox};
use eframe::egui;
use egui::{Checkbox, Ui};
use shared::model::EqConfig;
use state::{Action, FloatField, TypeField};

pub struct StingrayLpfView<'a, F: Fn(Action), G: Fn()> {
    config: &'a EqConfig,
    is_enabled: &'a bool,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> StingrayLpfView<'a, F, G> {
    pub fn new(config: &'a EqConfig, is_enabled: &'a bool, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            is_enabled,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for StingrayLpfView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config,
            is_enabled,
            dispatch,
            on_release,
        } = self;

        let on_release = || on_release();
        outer_frame().inner_margin(10.0).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Low Pass Filter");
                ui.add_space(4.0);
            });

            ui.horizontal(|ui| {
                add_knob(
                ui,
                styled_knob(
                    "Freq",
                    config.fc,
                    |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                    20.0..=20000.0,
                )
                .with_neutral(2000.0),
                on_release,
            );
            ui.add_space(10.0);
            add_knob(
                ui,
                styled_knob(
                    "Q",
                    config.q,
                    |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                    0.1..=100.0,
                )
                .with_neutral(1.0),
                on_release,
            );
            ui.add_space(10.0);
            checkbox(
                ui,
                |value| Checkbox::new(value, "Loop"),
                **is_enabled,
                |it| dispatch(Action::SetChild(TypeField::Mute(it))),
            );
            ui.add_space(105.0);
            });
        });
    }
}
