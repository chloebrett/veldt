use egui::{InnerResponse, Ui, Window};

use crate::app_state::DataState;

/// Creates a default window which is non-resizable, non-collapsible, and disables drag-to-scroll.
pub fn default_window(title: &str) -> egui::Window {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .drag_to_scroll(false)
}

/// New Type pattern to extend `egui::containers::Window::show` with logic to open and close window using a DataState enum.
pub struct StateWindow<'a>(pub Window<'a>);

impl StateWindow<'_> {
    pub fn show<R>(
        self,
        ui: &mut Ui,
        window_state: DataState,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        let StateWindow(window) = self;
        let mut open = window_state.get_value::<bool>(ui).unwrap_or(false);
        let response = window.open(&mut open).show(ui.ctx(), add_contents);
        if !open {
            window_state.set_value(ui, false);
        }
        response
    }
}
