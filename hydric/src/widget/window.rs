use crate::app_state::DataState;

/// Creates a default window which is non-resizable, non-collapsible, and disables drag-to-scroll.
pub fn default_window(title: &str) -> egui::Window {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .drag_to_scroll(false)
}

/// Extend `egui::containers::Window::show` with logic to open and close window using a DataState enum.
pub fn window_state_show<R>(
    ui: &mut egui::Ui,
    window_state: DataState,
    window: egui::Window,
    ctx: &egui::Context,
    show: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<egui::InnerResponse<Option<R>>> {
    let mut open = window_state.get_value::<bool>(ui).unwrap_or(false);
    let response = window.open(&mut open).show(ctx, show);
    if !open {
        window_state.set_value(ui, false);
    }
    response
}
