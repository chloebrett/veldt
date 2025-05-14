use crate::playback::AudioPlayer;
use crate::view::View;
use crate::widget::{get_set, selectable_value};
use egui::Ui;
use shared::model::{NoiseConfig, NoiseType};
use state::{Action, GeneratorSelector, TypeField};
use strum::IntoEnumIterator;

pub struct NoiseView<'a, F: Fn(Action), G: Fn()> {
    selector: GeneratorSelector,  // NOTE: could be used in the future for configuration/multiple noise generators
    config: &'a NoiseConfig,
    player: &'a mut AudioPlayer,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> NoiseView<'a, F, G> {
    pub fn new(
        selector: GeneratorSelector,
        config: &'a NoiseConfig,
        player: &'a mut AudioPlayer,
        dispatch: F,
        on_release: G,
    ) -> Self {
        Self {
            selector,
            config,
            player,
            dispatch,
            on_release,
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
}

impl<F: Fn(Action), G: Fn()> View for NoiseView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.noise_combo_box(ui);
            });
        });
    }
}
