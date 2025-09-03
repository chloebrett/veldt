use super::EnvelopeView;
use crate::playback::AudioPlayer;
use crate::view::View;
use crate::widget::{add_typable_knob, get_set, int_slider, selectable_value, styled_knob};
use egui::{Button, Sense, Ui};
use shared::model::{
    AntiAliasingMode, PitchName, PolyphonyMode, ScaleValue, SimpleWaveConfig, WaveType,
};
use state::{Action, FloatField, GeneratorSelector, TypeField, UintField};
use strum::IntoEnumIterator;

pub struct SimpleWaveView<'a, F: Fn(Action), G: Fn()> {
    selector: GeneratorSelector,
    config: &'a SimpleWaveConfig,
    player: &'a mut AudioPlayer,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> SimpleWaveView<'a, F, G> {
    pub fn new(
        selector: GeneratorSelector,
        config: &'a SimpleWaveConfig,
        player: &'a mut AudioPlayer,
        dispatch: F,
        on_release: G,
    ) -> Self {
        Self {
            selector,
            config,
            player,
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

    fn polyphony_combo_box(&self, ui: &mut Ui) {
        egui::ComboBox::from_label("Polyphony mode")
            .selected_text(self.config.polyphony_mode.to_string())
            .show_ui(ui, |ui| {
                for mode in PolyphonyMode::iter() {
                    selectable_value(
                        ui,
                        get_set(self.config.polyphony_mode, |it| {
                            (self.dispatch)(Action::SetChild(TypeField::PolyphonyMode(it)))
                        }),
                        mode,
                        mode.to_string(),
                    );
                }
            });

        // Only show polyphony limit if in polyphonic mode.
        if let PolyphonyMode::Polyphonic = self.config.polyphony_mode {
            int_slider(
                ui,
                "Polyphony limit",
                self.config.polyphony_limit as f64,
                |it| (self.dispatch)(Action::SetUint(UintField::PolyphonyLimit, it as u32)),
                0..=8,
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

                let detune_knob = styled_knob(
                    self.config.detune_cents,
                    |it| (self.dispatch)(Action::SetFloat(FloatField::Detune, it)),
                    0.0..=100.0,
                )
                .with_neutral(10.0);
                add_typable_knob(
                    ui,
                    detune_knob,
                    "Osc detune (cents)",
                    self.config.detune_cents,
                    |it| (self.dispatch)(Action::SetFloat(FloatField::Detune, it)),
                    0.0..=100.0,
                    &self.on_release,
                    40.0,
                );
                self.aliasing_combo_box(ui);
                self.polyphony_combo_box(ui);
            });

            if cfg!(feature = "extra_debug") {
                let response = ui.add(Button::new("[debug] Send note").sense(Sense::drag()));
                let pitch = PitchName {
                    octave: 4,
                    scale_value: ScaleValue::A,
                };
                if response.drag_started() {
                    self.player.send_note_on(self.selector, pitch);
                } else if response.drag_stopped() {
                    self.player.send_note_off(self.selector, pitch);
                }
            }

            ui.separator();
            ui.vertical(|ui| {
                EnvelopeView::new(&self.config.envelope, &self.dispatch, &self.on_release).ui(ui);
            })
        });
    }
}
