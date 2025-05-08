use super::SimpleWaveVisualiser;
use crate::view::View;
use crate::widget::{get_set, int_slider, knob, selectable_value};
use eframe::egui;
use egui::{Color32, Ui};
use shared::model::{LpfConfig};
use state::{Action, FloatField, TypeField, UintField};
use strum::IntoEnumIterator;

pub struct LpfView<'a, F: Fn(Action), G: Fn()> {
    config: &'a LpfConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> LpfView<'a, F, G> {
    pub fn new(
        config: &'a LpfConfig,
        dispatch: F,
        on_release: G,
    ) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for LpfView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            config,
            ref dispatch,
            ref on_release,
        } = *self;

        fn draw_lpf_controls<F, G>(
            ui: &mut Ui,
            config: &LpfConfig,
            dispatch: &F,
            on_release: &G,
        ) where
            F: Fn(Action),
            G: Fn(),
        {
            knob(
                ui,
                "Freq",
                config.fc,
                |it| dispatch(Action::SetFloat(FloatField::Fc, it)),
                20.0..=20000.0, 
                /* neutral= */ 2000.0,
                &self.on_release,
            );
    
            knob(
                ui,
                "Q",
                config.q,
                |it| dispatch(Action::SetFloat(FloatField::Q, it)),
                0.1..=100.0, // TODO: logarithmic
                /* neutral= */ 1.0,
                &self.on_release,
            );
    
            knob(
                ui,
                "Gain",
                config.gain,
                |it| dispatch(Action::SetFloat(FloatField::Gain, it)),
                -60.0..=60.0,
                /* neutral= */ 0.0,
                &self.on_release,
            );
    
            let eq_type = config.kind.clone();
            egui::ComboBox::from_label("EQ type")
                .selected_text(eq_type.to_string())
                .show_ui(ui, |ui| {
                    for eq_type in EqType::iter() {
                        selectable_value(
                            ui,
                            get_set(config.kind.clone(), |it| {
                                dispatch(Action::SetChild(TypeField::EqType(it)))
                            }),
                            eq_type.clone(),
                            eq_type.to_string(),
                        );
                    }
                });
            }

    }
}
