use super::SimpleWaveVisualiser;
use crate::widget::{knob, int_slider, get_set, selectable_value};
use eframe::egui;
use egui::{Color32, ComboBox};
use shared::model::{WaveType, OscillatorConfig};
use state::{Action, FloatField, UintField, TypeField};

pub fn subsynth_oscillator<F, G>(config: &OscillatorConfig, ui: &mut egui::Ui, dispatch: &F, on_release: &G, line_colour: Color32, fill_colour: Color32) -> egui::Response
    where
        F: Fn(Action),
        G: Fn(),
    {
        let frame = egui::Frame::new()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60)))
            .corner_radius(8.0)
            .inner_margin(6.0);

        frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {

                    // osc selection frame shows dropdown and visualiser for wave type
                    let osc_selection_frame = egui::Frame::new()
                        .fill(Color32::from_rgb(30, 30, 30))
                        .corner_radius(8.0)
                        .inner_margin(10.0);
                    osc_selection_frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let wave_type_str = match config.wave {
                                    WaveType::Sine => "Sine",
                                    WaveType::Square => "Square",
                                    WaveType::Saw => "Saw",
                                    WaveType::Triangle => "Triangle",
                                };
    
                                ComboBox::from_label("")
                                    .selected_text(wave_type_str)
                                    .show_ui(ui, |ui| {
                                        selectable_value(
                                            ui,
                                            get_set(config.wave, |_wave_type| {
                                                dispatch(Action::SetChild(TypeField::Wave(WaveType::Sine)))
                                            }),
                                            WaveType::Sine,
                                            WaveType::Sine.to_string(),
                                        );
                                        selectable_value(
                                            ui,
                                            get_set(config.wave, |_wave_type| {
                                                dispatch(Action::SetChild(TypeField::Wave(WaveType::Square)))
                                            }),
                                            WaveType::Square,
                                            WaveType::Square.to_string(),
                                        );
                                        selectable_value(
                                            ui,
                                            get_set(config.wave, |_wave_type| {
                                                dispatch(Action::SetChild(TypeField::Wave(WaveType::Saw)))
                                            }),
                                            WaveType::Saw,
                                            WaveType::Saw.to_string(),
                                        );
                                        selectable_value(
                                            ui,
                                            get_set(config.wave, |_wave_type| {
                                                dispatch(Action::SetChild(TypeField::Wave(WaveType::Triangle)))
                                            }),
                                            WaveType::Triangle,
                                            WaveType::Triangle.to_string(),
                                        );
                                    }
                                );
                            });
                            ui.add_space(10.0);
                            let visualiser = SimpleWaveVisualiser::new(
                                config.wave,
                                line_colour,
                                fill_colour,
                            );
    
                            visualiser.show(ui);
                        });
                    });

                    // volume, pan, tuning section
                    let four_knob_frame = egui::Frame::new()
                        .fill(Color32::from_rgb(30, 30, 30))
                        .corner_radius(8.0)
                        .inner_margin(10.0);
                    four_knob_frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            knob(
                                ui,
                                "Volume",
                                config.volume, // placeholder value fix this to be config instead
                                |it| dispatch(Action::SetFloat(FloatField::Volume, it)), // this action is prob not the right action idk
                                0.0..=1.0,
                                &on_release,
                            );
                            ui.add_space(6.0);
    
                            knob(
                                ui,
                                "Pan",
                                config.pan, // placeholder value
                                |it| dispatch(Action::SetFloat(FloatField::Pan, it)), // TODO fix action its a placeholder rn
                                0.0..=1.0,
                                &on_release,
                            );
                            ui.add_space(6.0);
    
                            knob(
                                ui,
                                "Coarse",
                                config.coarse_detune, // placeholder value
                                |it| dispatch(Action::SetFloat(FloatField::Detune, it)), // TODO fix action its a placeholder rn
                                0.0..=1.0,
                                &on_release,
                            );
                            ui.add_space(6.0);
    
                            knob(
                                ui,
                                "Fine",
                                config.fine_detune, // placeholder value
                                |it| dispatch(Action::SetFloat(FloatField::Detune, it)), // TODO fix action its a placeholder rn
                                0.0..=1.0,
                                &on_release,
                            );
                        });
                    });

                    // stacking section
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
                                1.0 as f64, // TODO placeholder value remember to hook it up to config instead
                                |it| dispatch(Action::SetUint(UintField::OscCount, it as u32)), // need diff action probably
                                1..=24,
                                &on_release,
                            );
                            ui.add_space(6.0);
                            knob(
                                ui,
                                "Osc detune",
                                0.0, // TODO placeholder value remember to hook it up to config instead
                                |it| dispatch(Action::SetFloat(FloatField::Detune, it)), // need diff action probably
                                0.0..=100.0,
                                &on_release,
                            );
                        });

                        // LFO and ENV selection section
                        let lfo_env_frame = egui::Frame::new()
                            .fill(Color32::from_rgb(30, 30, 30))
                            .corner_radius(8.0)
                            .inner_margin(10.0);
                        lfo_env_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // TODO add LFOS and ENV drop downs + knobs
                            })
                        })
                    });
                });
            })
            .response
    }

