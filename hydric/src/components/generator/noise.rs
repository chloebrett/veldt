use crate::view::View;
use crate::widget::{get_set, selectable_value};
use egui::Ui;
use shared::model::{NoiseConfig, NoiseType};
use state::{Action, TypeField};
use strum::IntoEnumIterator;

pub struct NoiseView<'a, F: Fn(Action)> {
    config: &'a NoiseConfig,
    dispatch: F,
}

impl<'a, F: Fn(Action)> NoiseView<'a, F> {
    pub fn new(config: &'a NoiseConfig, dispatch: F) -> Self {
        Self { config, dispatch }
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
}

impl<F: Fn(Action)> View for NoiseView<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.noise_combo_box(ui);
            });
        });
    }
}
