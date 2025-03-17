use egui::Ui;
use shared::model::EqType;
use shared::model::{EffectMeta, EqConfig};
use strum::IntoEnumIterator;
use std::cell::RefMut;

pub fn eq_control(config: RefMut<'_, EqConfig>, meta: RefMut<'_, EffectMeta>, ui: &mut Ui) {
    ui.label("Equalizer");
    ui.add(
        egui::Slider::new(&mut config.fc, 20.0..=20000.0)
            .text("Resonant frequency")
            .logarithmic(true),
    );
    ui.add(
        egui::Slider::new(&mut config.q, 0.1..=100.0)
            .text("Q value")
            .logarithmic(true),
    );
    let eq_type = config.kind.clone();
    egui::ComboBox::from_label("EQ type")
        .selected_text(format!("{}", eq_type))
        .show_ui(ui, |ui| {
            for eq_type in EqType::iter() {
                ui.selectable_value(&mut config.kind, eq_type.clone(), eq_type.to_string());
            }
        });
    ui.add(egui::Slider::new(&mut meta.wet, 0.0..=1.0).text("EQ wet"));
}
