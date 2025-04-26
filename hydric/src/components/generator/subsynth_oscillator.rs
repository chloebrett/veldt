use super::SimpleWaveVisualiser;
use crate::widget::{get_set, int_slider, knob, selectable_value};
use eframe::egui;
use egui::{Color32, Ui};
use shared::model::{OscillatorConfig, WaveType};
use state::{Action, FloatField, TypeField, UintField};
use strum::IntoEnumIterator;

pub fn subsynth_oscillator<F, G>(
    config: &OscillatorConfig,
    ui: &mut egui::Ui,
    dispatch: &F,
    on_release: &G,
    line_colour: Color32,
    fill_colour: Color32,
) -> egui::Response
where
    F: Fn(Action),
    G: Fn(),
{
    fn draw_wave_selection<F>(
        ui: &mut Ui,
        config: &OscillatorConfig,
        dispatch: &F,
        line_colour: Color32,
        fill_colour: Color32,
    ) where
        F: Fn(Action),
    {
        let osc_selection_frame = egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 30))
            .corner_radius(8.0)
            .inner_margin(10.0);
        osc_selection_frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("")
                        .selected_text(config.wave.to_string())
                        .show_ui(ui, |ui| {
                            for wave in WaveType::iter() {
                                selectable_value(
                                    ui,
                                    get_set(config.wave, |wave_type| {
                                        dispatch(Action::SetChild(TypeField::Wave(wave_type)))
                                    }),
                                    wave,
                                    wave.to_string(),
                                );
                            }
                        });
                });
                ui.add_space(10.0);
                let visualiser = SimpleWaveVisualiser::new(config.wave, line_colour, fill_colour);

                visualiser.show(ui);
            });
        });
    }

    fn draw_oscillator_controls<F, G>(
        ui: &mut Ui,
        config: &OscillatorConfig,
        dispatch: &F,
        on_release: &G,
    ) where
        F: Fn(Action),
        G: Fn(),
    {
        let four_knob_frame = egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 30))
            .corner_radius(8.0)
            .inner_margin(10.0);
        four_knob_frame.show(ui, |ui| {
            ui.vertical(|ui| {
                knob(
                    ui,
                    "Volume",
                    config.volume,
                    |it| dispatch(Action::SetFloat(FloatField::Volume, it)),
                    0.0..=1.0,
                    0.0,
                    on_release,
                );
                ui.add_space(2.0);

                knob(
                    ui,
                    "Pan",
                    config.pan,
                    |it| dispatch(Action::SetFloat(FloatField::Pan, it)),
                    -1.0..=1.0,
                    0.0,
                    on_release,
                );
                ui.add_space(2.0);

                knob(
                    ui,
                    "Coarse",
                    config.osc_detune.floor(),
                    |it| dispatch(Action::SetFloat(FloatField::Detune, it)),
                    -24.0..=24.0,
                    0.0,
                    on_release,
                );
                ui.add_space(2.0);

                knob(
                    ui,
                    "Fine",
                    config.osc_detune % 1.0,
                    |it| dispatch(Action::SetFloat(FloatField::Detune, it)),
                    -100.0..=100.0,
                    0.0,
                    on_release,
                );
            });
        });
    }

    fn draw_stacking_controls<F, G>(
        ui: &mut Ui,
        config: &OscillatorConfig,
        dispatch: &F,
        on_release: &G,
    ) where
        F: Fn(Action),
        G: Fn(),
    {
        let stacking_frame = egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 30))
            .corner_radius(8.0)
            .inner_margin(10.0);
        stacking_frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Stacking");
                ui.add_space(4.0);
                int_slider(
                    ui,
                    "Unison",
                    config.osc_count as f64,
                    |it| dispatch(Action::SetUint(UintField::OscCount, it as u32)),
                    1..=24,
                    on_release,
                );
                ui.add_space(6.0);
                knob(
                    ui,
                    "Unison Detune",
                    config.unison_detune,
                    |it| dispatch(Action::SetFloat(FloatField::Detune, it)),
                    0.0..=100.0,
                    0.0,
                    on_release,
                );
            });
        });
    }

    let frame = egui::Frame::new()
        .fill(Color32::from_rgb(50, 50, 50))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60)))
        .corner_radius(8.0)
        .inner_margin(6.0);

    frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                draw_wave_selection(ui, config, dispatch, line_colour, fill_colour);

                draw_oscillator_controls(ui, config, dispatch, on_release);

                draw_stacking_controls(ui, config, dispatch, on_release);
            });
        })
        .response
}
