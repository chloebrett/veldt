use crate::{app_state::WindowState, view::View};
use egui::{Button, Ui, menu::bar};
use state::Store;

pub struct Menu<'a> {
    store: &'a mut Store,
    window_state: &'a mut WindowState,
}

impl<'a> Menu<'a> {
    pub fn new(store: &'a mut Store, window_state: &'a mut WindowState) -> Self {
        Menu {
            store,
            window_state,
        }
    }
}

impl View for Menu<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = &mut self.store;
        let window_state = &mut self.window_state;
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
                let mut button_with_tick = |label, state: &mut bool| {
                    let suffix = if *state { " ✅" } else { "" };
                    if ui.button(format!("{}{}", label, suffix)).clicked() {
                        *state ^= true
                    }
                };
                button_with_tick("Mixers", &mut window_state.mixer.visible);
                button_with_tick("Generators", &mut window_state.generator_list);
                button_with_tick("Scale", &mut window_state.scale);
                button_with_tick("Samples", &mut window_state.sample_tree);
                button_with_tick("Track Roll", &mut window_state.track_roll);
            });
            ui.menu_button("Effects", |ui| if ui.button("Add effect").clicked() {});
            ui.menu_button(
                "Generators",
                |ui| {
                    if ui.button("Add generator").clicked() {}
                },
            );
            let sample_response = ui.add(Button::new("📂").selected(window_state.sample_tree));
            if sample_response.clicked() {
                window_state.sample_tree ^= true;
            }
            sample_response.on_hover_ui(|ui| {
                ui.label("Samples");
            });
            let sound_response = ui.add(Button::new("🎷"));
            sound_response.on_hover_ui(|ui| {
                ui.label("Sound library");
            });
            let effect_response = ui.add(Button::new("🎨").selected(window_state.mixer.visible));
            if effect_response.clicked() {
                window_state.mixer.visible ^= true;
            }
            effect_response.on_hover_ui(|ui| {
                ui.label("Effects/Mixers");
            });
            let track_response = ui.add(Button::new("📄").selected(window_state.track_roll));
            if track_response.clicked() {
                window_state.track_roll ^= true;
            }
            track_response.on_hover_ui(|ui| {
                ui.label("Track Roll");
            })
        });
    }
}
