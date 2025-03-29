use super::WindowState;
use egui::Ui;

pub fn window_toggle_button(toggle_value: bool, ui: &mut Ui, toggle_name: String) -> bool {
    let button_text = if toggle_value {
        format!("Hide {}", toggle_name).to_string()
    } else {
        format!("Show {}", toggle_name).to_string()
    };
    if ui.button(button_text).clicked() {
        !toggle_value
    } else {
        toggle_value
    }
}

pub fn toggle_window_panel(window_state: &mut WindowState, ui: &mut Ui) {
    ui.vertical(|ui| {
        window_state.show_effects =
            window_toggle_button(window_state.show_effects, ui, "Effects".to_string());
        window_state.show_envelope =
            window_toggle_button(window_state.show_envelope, ui, "Envelope".to_string());
    });
    ui.vertical(|ui| {
        window_state.show_generator =
            window_toggle_button(window_state.show_generator, ui, "Generator".to_string());
        window_state.show_scale =
            window_toggle_button(window_state.show_scale, ui, "Scale".to_string());
    });
}
