use crate::view::View;
use crate::widget::{add_typable_knob, checkbox, outer_frame, styled_knob};
use eframe::egui;
use egui::{Checkbox, Ui};
use shared::model::EqConfig;
use state::{Action, FloatField, TypeField};

pub struct StingrayLpfView<'a, F: Fn(Action), G: Fn(Action), H: Fn()> {
    config: &'a EqConfig,
    lpf_on: bool,
    dispatch: F,
    gen_dispatch: G,
    on_release: H,
}

impl<'a, F: Fn(Action), G: Fn(Action), H: Fn()> StingrayLpfView<'a, F, G, H> {
    pub fn new(
        config: &'a EqConfig,
        lpf_on: bool,
        dispatch: F,
        gen_dispatch: G,
        on_release: H,
    ) -> Self {
        Self {
            config,
            lpf_on,
            dispatch,
            gen_dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn(Action), H: Fn()> View for StingrayLpfView<'_, F, G, H> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config,
            lpf_on,
            dispatch,
            gen_dispatch,
            on_release,
        } = self;

        let on_release = || on_release();
        outer_frame().inner_margin(10.0).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Low Pass Filter");
                ui.add_space(4.0);
            });

            ui.horizontal(|ui| {
                let freq_knob = styled_knob(
                    config.fc,
                    |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                    20.0..=20000.0,
                )
                .with_neutral(2000.0);
                let q_knob = styled_knob(
                    config.q,
                    |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                    0.1..=100.0,
                )
                .with_neutral(1.0);
                add_typable_knob(
                    ui,
                    freq_knob,
                    "Freq",
                    config.fc,
                    |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                    20.0..=20000.0,
                    &on_release,
                );
                ui.add_space(10.0);
                add_typable_knob(
                    ui,
                    q_knob,
                    "Q",
                    config.q,
                    |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                    0.1..=100.0,
                    &on_release,
                );
                ui.add_space(10.0);
                checkbox(
                    ui,
                    |value| Checkbox::new(value, "Enable"),
                    *lpf_on,
                    |it| gen_dispatch(Action::SetChild(TypeField::LpfOn(it))),
                );
                ui.add_space(25.0);
            });
        });
    }
}
