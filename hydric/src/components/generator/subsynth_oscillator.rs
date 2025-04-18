use super::SimpleWaveVisualiser;
use crate::widget::{knob, int_slider, get_set};
use eframe::egui;
use egui::{Color32, ComboBox};
use shared::model::WaveType;
use state::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OscillatorId {
    Osc1,
    Osc2,
    Osc3,
}

// TODO fix this make sure its hooked up to config instead
pub struct SubsynthOscillator {
    id: OscillatorId,
    wave_type: WaveType,
    line_color: Color32,
    fill_color: Color32,
    // for styling
    border_color: Color32,
    border_width: f32,
    border_radius: f32,
    background_color: Color32,
}

impl SubsynthOscillator {
    pub fn new(
        id: OscillatorId,
        wave_type: WaveType,
        line_color: Color32,
        fill_color: Color32,
    ) -> Self {
        Self {
            id,
            wave_type,
            line_color,
            fill_color,
            // Default styling
            border_color: Color32::from_rgb(60, 60, 60),
            border_width: 1.0,
            border_radius: 8.0,
            background_color: Color32::from_rgb(50, 50, 50),
        }
    }

    // Method to show the component in the UI
    pub fn show<F, G>(&mut self, ui: &mut egui::Ui, dispatch: F, on_release: G) -> egui::Response
    where
        F: Fn(Action),
        G: Fn(),
    {
        let frame = egui::Frame::new()
            .fill(self.background_color)
            .stroke(egui::Stroke::new(self.border_width, self.border_color))
            .corner_radius(self.border_radius)
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
                                let wave_type_str = match self.wave_type {
                                    WaveType::Sine => "Sine",
                                    WaveType::Square => "Square",
                                    WaveType::Saw => "Saw",
                                    WaveType::Triangle => "Triangle",
                                };
    
                                ComboBox::from_label("")
                                    .selected_text(wave_type_str)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.wave_type,
                                            WaveType::Sine,
                                            "Sine",
                                        );
                                        ui.selectable_value(
                                            &mut self.wave_type,
                                            WaveType::Square,
                                            "Square",
                                        );
                                        ui.selectable_value(&mut self.wave_type, WaveType::Saw, "Saw");
                                        ui.selectable_value(
                                            &mut self.wave_type,
                                            WaveType::Triangle,
                                            "Triangle",
                                        );
                                    });
                            });
                            ui.add_space(10.0);
                            let visualiser = SimpleWaveVisualiser::new(
                                self.wave_type,
                                self.line_color,
                                self.fill_color,
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
                                0.0, // placeholder value fix this to be config instead
                                |it| dispatch(Action::SetGeneratorVolume(it)), // this action is prob not the right action idk
                                0.0..=1.0,
                                &on_release,
                            );
                            ui.add_space(6.0);
    
                            knob(
                                ui,
                                "Pan",
                                0.0, // placeholder value
                                |it| dispatch(Action::SetGeneratorVolume(it)), // TODO fix action its a placeholder rn
                                0.0..=1.0,
                                &on_release,
                            );
                            ui.add_space(6.0);
    
                            knob(
                                ui,
                                "Coarse",
                                0.0, // placeholder value
                                |it| dispatch(Action::SetGeneratorVolume(it)), // TODO fix action its a placeholder rn
                                0.0..=1.0,
                                &on_release,
                            );
                            ui.add_space(6.0);
    
                            knob(
                                ui,
                                "Fine",
                                0.0, // placeholder value
                                |it| dispatch(Action::SetGeneratorVolume(it)), // TODO fix action its a placeholder rn
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
                                |it| dispatch(Action::SetOscCount(it as u32)), // need diff action probably
                                1..=24,
                                &on_release,
                            );
                            ui.add_space(6.0);
                            knob(
                                ui,
                                "Osc detune",
                                0.0, // TODO placeholder value remember to hook it up to config instead
                                |it| dispatch(Action::SetDetuneCents(it)), // need diff action probably
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

    // Getter for current wave type
    pub fn wave_type(&self) -> WaveType {
        self.wave_type
    }
    // gett for osc id
    pub fn id(&self) -> OscillatorId {
        self.id
    }
}
