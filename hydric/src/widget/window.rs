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
        self.show_with_closure(
            ui,
            window_state.get_value::<bool>(ui).unwrap_or(false),
            move |ui| window_state.set_value(ui, false),
            add_contents,
        )
    }

    /// Shows a window that calls a closure when it is closed.
    pub fn show_with_closure<R>(
        self,
        ui: &mut Ui,
        show: bool,
        on_close: impl Fn(&Ui),
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
