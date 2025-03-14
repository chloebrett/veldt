use egui::Ui;
use shared::model::{SimpleWaveConfig, WaveType};

pub fn generator_control(config: &mut SimpleWaveConfig, ui: &mut Ui) {
    egui::ComboBox::from_label("Wave type")
        .selected_text(config.wave.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut config.wave, WaveType::Sine, "Sine");
            ui.selectable_value(&mut config.wave, WaveType::Square, "Square");
            ui.selectable_value(&mut config.wave, WaveType::Saw, "Saw");
            ui.selectable_value(&mut config.wave, WaveType::Triangle, "Triangle");
        });
    ui.add(
        egui::Slider::new(&mut config.osc_count, 1..=24)
            .text("Osc count")
            .logarithmic(true),
    );
    ui.add(
        egui::Slider::new(&mut config.detune_cents, 0.0..=100.0)
            .text("Osc detune")
            .logarithmic(true),
    );
}
