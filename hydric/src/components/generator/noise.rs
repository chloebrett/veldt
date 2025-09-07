use crate::view::View;
use crate::widget::{get_set, selectable_value};
use egui::Ui;
use shared::model::{GeneratorMeta, NoiseConfig, NoiseType};
use state::{Action, TypeField};
use strum::IntoEnumIterator;

pub struct NoiseView<'a, F: Fn(Action)> {
    config: &'a NoiseConfig,
    dispatch: F,
    meta: &'a GeneratorMeta,
}

impl<'a, F: Fn(Action)> NoiseView<'a, F> {
    pub fn new(config: &'a NoiseConfig, dispatch: F, meta: &'a GeneratorMeta) -> Self {
        Self {
            config,
            dispatch,
            meta,
        }
    }

    fn noise_combo_box(&self, ui: &mut Ui) {
        egui::ComboBox::from_label("Noise type")
            .selected_text(self.config.kind.to_string())
            .show_ui(ui, |ui| {
                for noise_type in NoiseType::iter() {
                    selectable_value(
                        ui,
                        get_set(self.config.kind, |it| {
                            (self.dispatch)(Action::SetChild(TypeField::NoiseType(it)))
                        }),
                        noise_type,
                        noise_type.to_string(),
                    );
                }
            });
    }

    fn generator_name(&self, ui: &mut Ui) {
        ui.label("Rename Generator/Instrument Name");
        ui.add_space(2.0);
        let mut name = self.meta.name.clone();
        let response = ui.text_edit_singleline(&mut name);
        if response.changed() {
            (self.dispatch)(Action::SetChild(TypeField::GeneratorName(name.to_string())));
        }
    }
}

impl<F: Fn(Action)> View for NoiseView<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.generator_name(ui);
                self.noise_combo_box(ui);
            });
        });
    }
}
