use crate::WindowState;
use egui::Ui;

pub fn window_toggle_button(toggle_value: bool, ui: &mut Ui, toggle_name: &str) -> bool {
    toggle_value || ui.button(toggle_name.to_string()).clicked()
}

pub fn toggle_window_panel(window_state: &mut WindowState, ui: &mut Ui) {
    ui.horizontal(|ui| {
        window_state.mixer.visible = window_toggle_button(window_state.mixer.visible, ui, "Mixer");
        window_state.generator_list =
            window_toggle_button(window_state.generator_list, ui, "Generators");
        window_state.scale = window_toggle_button(window_state.scale, ui, "Scale");
        window_state.sample_tree = window_toggle_button(window_state.sample_tree, ui, "Samples");
    });
}
