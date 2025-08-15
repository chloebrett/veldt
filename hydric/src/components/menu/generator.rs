use egui::Ui;
use shared::model::{Generator, GeneratorInstance, GeneratorMeta};
use state::{Action, GeneratorSelector, Store, TypeField};
use strum::IntoEnumIterator;

use crate::{
    components::generator::generator_name, local_state::LocalState, view::View,
    window_state::WindowState,
};

pub struct GeneratorMenuOptions<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> GeneratorMenuOptions<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }
}

impl View for GeneratorMenuOptions<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.menu_button("Add new", |ui| {
            for generator in Generator::iter() {
                let temp_instance = GeneratorInstance {
                    it: generator.clone(),
                    meta: GeneratorMeta::default(),
                };
                ui.menu_button(generator_name(&temp_instance.clone()), |ui| {
                    for generator_id in self.store.get().project.generators.keys() {
                        if ui.button(format!("Generator {:?}", generator_id)).clicked() {
                            let instance = GeneratorInstance {
                                it: generator.clone(),
                                meta: GeneratorMeta::default(),
                            };
                            let generator_sel = GeneratorSelector(*generator_id);
                            self.store.dispatch(
                                &generator_sel,
                                Action::AddChild(TypeField::Generator(instance)),
                            );
                            // Open generators window.
                            // self.window_state.generators.set(generator_sel, true);
                            // self.window_state.generator_list = true;
                        }
                    }
                });
            }
        });
    }
}
