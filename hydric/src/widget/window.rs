/// Creates a default window which is non-resizable, non-collapsible, and disables drag-to-scroll.
pub fn default_window(title: &str) -> egui::Window {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .drag_to_scroll(false)
}
