use egui::Ui;
use shared::model::{DelayConfig, EffectMeta};

pub fn delay_control(config: &mut DelayConfig, meta: &mut EffectMeta, ui: &mut Ui) {
    ui.label("Delay");
    ui.add(egui::Slider::new(&mut config.amplitude, 0.0..=1.0).text("Delay amplitude"));
    ui.add(
        egui::Slider::new(&mut config.delay_ms, 1.0..=1000.0)
            .text("Delay ms")
            .logarithmic(true),
    );
    ui.add(egui::Slider::new(&mut meta.wet, 0.0..=1.0).text("Delay wet"));
}
