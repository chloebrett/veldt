use super::EnvelopeView;
use crate::view::View;
use crate::widget::{get_set, int_slider, knob, selectable_value};
use egui::{Button, Sense, Ui};
use shared::model::{AntiAliasingMode, SimpleWaveConfig, ScaleValue, WaveType, PitchName};
use state::{Action, FloatField, TypeField, UintField, GeneratorSelector};
use strum::IntoEnumIterator;
use crate::AudioState;

pub struct SimpleWaveView<'a, F: Fn(Action), G: Fn()> {
    selector: GeneratorSelector,
    config: &'a SimpleWaveConfig,
    audio_state: &'a AudioState,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SimpleWaveView<'a, F, G> {
    pub fn new(
        selector: GeneratorSelector,
        config: &'a SimpleWaveConfig,
        audio_state: &'a AudioState,
        dispatch: F,
        on_release: G,
    ) -> Self {
        Self {
            selector,
            config,
            audio_state,
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

            if cfg!(feature = "extra_debug") {
                let response = ui.add(Button::new("[debug] Send note").sense(Sense::drag()));
                let pitch = PitchName {
                        octave: 4,
                        scale_value: ScaleValue::A,
                    };
                if response.drag_started() {
                    self.audio_state.player.send_note_on(self.selector, pitch);
                } else if response.drag_stopped() {
                    self.audio_state.player.send_note_off(self.selector, pitch);
                }
            }

            ui.separator();
            ui.vertical(|ui| {
                EnvelopeView::new(&self.config.envelope, &self.dispatch, &self.on_release).ui(ui);
            })
        });
    }
}
