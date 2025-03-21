use crate::state::Store;
use egui::{Button, Ui};

pub fn undo_redo_control(store: &mut Store, ui: &mut Ui) {
    if ui
        .add_enabled(store.can_undo(), Button::new("Undo"))
        .clicked()
    {
        store.pend_undo();
    }
    if ui
        .add_enabled(store.can_redo(), Button::new("Redo"))
        .clicked()
    {
        store.pend_redo();
    }
}
