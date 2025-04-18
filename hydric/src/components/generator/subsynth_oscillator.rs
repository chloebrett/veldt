use super::SimpleWaveVisualiser;
use crate::widget::knob;
use eframe::egui;
use egui::{Color32, ComboBox, Slider};
use shared::model::WaveType;
use state::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OscillatorId {
    Osc1,
    Osc2,
    Osc3,
}

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
            border_color: Color32::GRAY, // change to black to match figma once the rest is implemented
            border_width: 1.0,
            border_radius: 8.0,
            background_color: Color32::from_rgb(30, 30, 30),
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
            .inner_margin(10.0);

        frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        // Wave type dropdown
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
                    ui.add_space(14.0);
                    ui.vertical(|ui| {
                        knob(
                            ui,
                            "Volume",
                            0.0, // placeholder value fix this to be config instead
                            |it| dispatch(Action::SetGeneratorVolume(it)), // this action is prob not the right action idk
                            0.0..=1.0,
                            &on_release,
                        );
                        ui.add_space(3.0);

                        knob(
                            ui,
                            "Pan",
                            0.0, // placeholder value
                            |it| dispatch(Action::SetGeneratorVolume(it)), // TODO fix action its a placeholder rn
                            0.0..=1.0,
                            &on_release,
                        );
                        ui.add_space(3.0);

                        knob(
                            ui,
                            "Coarse",
                            0.0, // placeholder value
                            |it| dispatch(Action::SetGeneratorVolume(it)), // TODO fix action its a placeholder rn
                            0.0..=1.0,
                            &on_release,
                        );
                        ui.add_space(3.0);

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
