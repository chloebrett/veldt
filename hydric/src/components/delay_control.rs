use super::app::App;
use egui::Ui;

pub fn delay_control(app: &mut App, ui: &mut Ui) {
    ui.add(egui::Slider::new(&mut app.delay_amplitude, 0.0..=1.0).text("Delay amplitude"));
    ui.add(
        egui::Slider::new(&mut app.delay_ms, 1.0..=1000.0)
            .text("Delay ms")
            .logarithmic(true),
    );
    ui.add(egui::Slider::new(&mut app.delay_wet, 0.0..=1.0).text("Delay wet"));
}
