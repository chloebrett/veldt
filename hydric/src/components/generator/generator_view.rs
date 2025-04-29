use super::simple_wave_control;
use super::subsynth::SubSynthView;
use crate::view::View;
use crate::widget::StateWindow;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use shared::model::GeneratorType;
use state::{Action, Selector, Store};

pub struct GeneratorView<'a, F: FnMut()> {
    store: &'a Store,
    generator_index: usize,
    visible: bool,
    on_close: F,
}

impl<'a, F: FnMut()> GeneratorView<'a, F> {
    pub fn new(store: &'a Store, generator_index: usize, visible: bool, on_close: F) -> Self {
        GeneratorView {
            store,
            generator_index,
            visible,
            on_close,
        }
    }
}

impl<F: FnMut()> View for GeneratorView<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        // TODO: reduce duplication of passing around indexes for e.g. generators as well as
        // selectors. Just have a unique object for each selector type and pass that around?
        let sel = Selector::Generator(self.generator_index);
        let generator = &self.store.get().project.generators[self.generator_index];

        let title = match &generator.kind {
            GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
            GeneratorType::Noise { .. } => "Noise Generator",
            GeneratorType::SubSynth { .. } => "Subtractive Synth",
        };

        StateWindow(default_window(title).default_pos(Pos2 { x: 1100.0, y: 20.0 }))
            .show_with_closure(
                ui,
                self.visible,
                |_| (self.on_close)(),
                |ui| {
                    let generator_type = generator.kind.clone();
                    let dispatch = |action| self.store.dispatch(&sel, action);
                    let on_release = || self.store.dispatchr(Action::Release);

                    match generator_type {
                        GeneratorType::SimpleWave { config } => {
                            simple_wave_control(&config, dispatch, on_release, ui)
                        }
                        GeneratorType::Noise { .. } => todo!(),
                        GeneratorType::SubSynth { config } => {
                            SubSynthView::new(&config, dispatch, on_release).ui(ui);
                        }
                    };
                },
            );
    }
}
