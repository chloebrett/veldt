use egui::{InnerResponse, Ui, Window};

/// Creates a default window which is non-resizable, non-collapsible, and disables drag-to-scroll.
pub fn default_window(title: &str) -> Window {
    Window::new(title)
        .collapsible(false)
        .resizable(false)
        .drag_to_scroll(false)
}

/// NewType pattern to extend `egui::containers::Window::show` with logic to open and close
/// window using arbitrary data.
pub struct StateWindow<'a>(pub Window<'a>);

impl StateWindow<'_> {
    /// Shows a window that calls a closure when it is closed.
    pub fn show_with_closure<R>(
        self,
        ui: &mut Ui,
        show: bool,
        on_close: impl FnOnce(&Ui),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        let StateWindow(window) = self;
        let mut open = show;
        let response = window.open(&mut open).show(ui.ctx(), add_contents);
        if open != show {
            on_close(ui);
        }
        response
    }
}
