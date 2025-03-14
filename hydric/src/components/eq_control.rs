use super::app::App;
use egui::Ui;
use shared::model::EqType;
use strum::IntoEnumIterator;

pub fn eq_control(app: &mut App, ui: &mut Ui) {
    ui.add(
        egui::Slider::new(&mut app.resonant_freq, 20.0..=20000.0)
            .text("Resonant frequency")
            .logarithmic(true),
    );
    ui.add(
        egui::Slider::new(&mut app.q, 0.1..=100.0)
            .text("Q value")
            .logarithmic(true),
    );
    egui::ComboBox::from_label("EQ type")
        .selected_text(format!("{}", app.eq_type))
        .show_ui(ui, |ui| {
            for eq_type in EqType::iter() {
                ui.selectable_value(&mut app.eq_type, eq_type.clone(), eq_type.to_string());
            }
        });
    ui.add(egui::Slider::new(&mut app.eq_wet, 0.0..=1.0).text("EQ wet"));
}
