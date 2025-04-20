use crate::view::View;
use egui::{Button, Ui, menu::bar};
use state::Store;

pub struct Menu<'a> {
    store: &'a mut Store,
}

impl<'a> Menu<'a> {
    pub fn new(store: &'a mut Store) -> Self {
        Menu { store }
    }
}

impl View for Menu<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = &mut self.store;
        bar(ui, |ui| {
            ui.label("Veldt");
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked() {}
                if ui.button("Load").clicked() {}
                if ui.button("Export").clicked() {}
            });
            ui.menu_button("Edit", |ui| {
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
            });
            ui.menu_button("Windows", |ui| {
                if ui.button("Mixers").clicked() {}
                if ui.button("Generators").clicked() {}
                if ui.button("Scale").clicked() {}
                if ui.button("Sample").clicked() {}
            });
            ui.menu_button("Effects", |ui| if ui.button("Add effect").clicked() {});
            ui.menu_button(
                "Generators",
                |ui| {
                    if ui.button("Add generator").clicked() {}
                },
            );
            ui.menu_button("📂", |_| {});
            ui.menu_button("🎷", |_| {});
            ui.menu_button("🎨", |_| {});
            ui.menu_button("📄", |_| {});
        });
    }
}
