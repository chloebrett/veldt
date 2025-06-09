use egui::{InnerResponse, Ui, Window};

use crate::window_state::{WindowKind, WindowState};

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

impl<'a> StateWindow<'a> {
    /// Configures and shows a window based on `WindowState` and `WindowKind`.
    // TODO: Move title onto `WindowState`.
    pub fn show_from_window_state<R>(
        ui: &mut Ui,
        window_state: &WindowState,
        window_kind: WindowKind,
        title: &'a str,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        Self(
            default_window(title)
                .id(window_state.get_id(window_kind))
                .default_pos(window_state.get_pos(window_kind)),
        )
        .show_with_closure(
            ui,
            window_state.get_visible(window_kind),
            move |_| window_state.set_visible(window_kind, false),
            add_contents,
        )
    }

    // TODO: factor this so that we can just pass in the window we want, rather than needing a new
    // method for the resizable version!
    pub fn show_from_window_state_resizable<R>(
        ui: &mut Ui,
        window_state: &WindowState,
        window_kind: WindowKind,
        title: &'a str,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        Self(
            default_window(title)
                .id(window_state.get_id(window_kind))
                .default_pos(window_state.get_pos(window_kind))
                .resizable(true),
        )
        .show_with_closure(
            ui,
            window_state.get_visible(window_kind),
            move |_| window_state.set_visible(window_kind, false),
            add_contents,
        )
    }

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
