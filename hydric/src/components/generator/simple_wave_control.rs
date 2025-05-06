use super::EnvelopeView;
use crate::view::View;
use crate::widget::{get_set, int_slider, knob, selectable_value};
use egui::Ui;
use shared::model::{AntiAliasingMode, SimpleWaveConfig, WaveType};
use state::{Action, FloatField, TypeField, UintField};
use strum::IntoEnumIterator;

pub struct SimpleWaveView<'a, F: Fn(Action), G: Fn()> {
    config: &'a SimpleWaveConfig,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SimpleWaveView<'a, F, G> {
    pub fn new(config: &'a SimpleWaveConfig, dispatch: F, on_release: G) -> Self {
        Self {
            config,
            dispatch,
            on_release,
        }
    }

    fn wave_combo_box(&self, ui: &mut Ui) {
        egui::ComboBox::from_label("Wave type")
            .selected_text(self.config.wave.to_string())
            .show_ui(ui, |ui| {
                for wave in WaveType::iter() {
                    selectable_value(
                        ui,
                        get_set(self.config.wave, |it| {
                            (self.dispatch)(Action::SetChild(TypeField::Wave(it)))
                        }),
                        wave,
                        wave.to_string(),
                    );
                }
            });
    }

    fn aliasing_combo_box(&self, ui: &mut Ui) {
        egui::ComboBox::from_label("Anti aliasing mode")
            .selected_text(self.config.anti_aliasing_mode.to_string())
            .show_ui(ui, |ui| {
                for mode in AntiAliasingMode::iter() {
                    selectable_value(
                        ui,
                        get_set(self.config.anti_aliasing_mode, |it| {
                            (self.dispatch)(Action::SetChild(TypeField::AntiAliasingMode(it)))
                        }),
                        mode,
                        mode.to_string(),
                    );
                }
            });

        // Only show oversample factor if the anti-aliasing mode is oversample.
        if let AntiAliasingMode::Oversample = self.config.anti_aliasing_mode {
            int_slider(
                ui,
                "Oversample factor",
                self.config.oversample_factor as f64,
                |it| (self.dispatch)(Action::SetUint(UintField::OversampleFactor, it as u32)),
                2..=10,
                &self.on_release,
            );
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for SimpleWaveView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.wave_combo_box(ui);
                int_slider(
                    ui,
                    "Unison",
                    self.config.osc_count as f64,
                    |it| (self.dispatch)(Action::SetUint(UintField::OscCount, it as u32)),
                    1..=24,
                    &self.on_release,
                );
                knob(
                    ui,
                    "Osc detune (cents)",
                    self.config.detune_cents,
                    |it| (self.dispatch)(Action::SetFloat(FloatField::Detune, it)),
                    0.0..=100.0,
                    /* neutral= */ 10.0,
                    &self.on_release,
                );
                self.aliasing_combo_box(ui);
            });
            // TODO: move this to the individual generator UI.
            ui.separator();
            ui.vertical(|ui| {
                EnvelopeView::new(&self.config.envelope, &self.dispatch, &self.on_release).ui(ui);
            })
        });
    }
}
