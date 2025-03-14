use super::app::App;
use egui::Ui;

pub fn window_toggle_button(toggle_value: bool, ui: &mut Ui, toggle_name: String) -> bool {
    let button_text = match toggle_value {
        true => String::from(format!("Hide {}", toggle_name)),
        false => String::from(format!("Show {}", toggle_name)),
    };
    if ui.button(button_text).clicked() {
        match toggle_value {
            true => false,
            false => true,
        }
    } else {
        toggle_value
    }
}

pub fn toggle_window_panel(app: &mut App, ui: &mut Ui) {
    ui.vertical(|ui| {
        app.show_effects = window_toggle_button(app.show_effects, ui, String::from("Effects"));
        app.show_envelope = window_toggle_button(app.show_envelope, ui, String::from("Envelope"));
    });
    ui.vertical(|ui| {
        app.show_generator =
            window_toggle_button(app.show_generator, ui, String::from("Generator"));
        app.show_scale = window_toggle_button(app.show_scale, ui, String::from("Scale"));
    });
}
