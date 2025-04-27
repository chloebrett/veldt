use crate::view::View;
use egui::{Button, Ui};
use state::Store;

pub struct UndoRedoView<'a> {
    store: &'a mut Store,
}

impl<'a> UndoRedoView<'a> {
    pub fn new(store: &'a mut Store) -> Self {
        UndoRedoView { store }
    }
}

impl View for UndoRedoView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        if ui
            .add_enabled(self.store.can_undo(), Button::new("Undo"))
            .clicked()
        {
            self.store.pend_undo();
        }
        if ui
            .add_enabled(self.store.can_redo(), Button::new("Redo"))
            .clicked()
        {
            self.store.pend_redo();
        }
    }
}
