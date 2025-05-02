use super::simple_wave_control::SimpleWaveView;
use super::subsynth::SubSynthView;
use crate::view::View;
use crate::widget::StateWindow;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use shared::model::Generator;
use state::{Action, GeneratorSelector, Store};

pub struct GeneratorView<'a, F: FnMut()> {
    store: &'a Store,
    selector: &'a GeneratorSelector,
    visible: bool,
    on_close: F,
}

impl<'a, F: FnMut()> GeneratorView<'a, F> {
    pub fn new(
        store: &'a Store,
        selector: &'a GeneratorSelector,
        visible: bool,
        on_close: F,
    ) -> Self {
        Self {
            store,
            selector,
            visible,
            on_close,
        }
    }
}

impl<F: FnMut()> View for GeneratorView<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        let instance = &self.store.select(self.selector);

        let title = match &instance.it {
            Generator::SimpleWave(_) => "Simple Wave Generator",
            Generator::Noise(_) => "Noise Generator",
            Generator::SubSynth(_) => "Subtractive Synth",
        };

        StateWindow(default_window(title).default_pos(Pos2 { x: 1100.0, y: 20.0 }))
            .show_with_closure(
                ui,
                self.visible,
                |_| (self.on_close)(),
                |ui| {
                    let generator = instance.it.clone();
                    let dispatch = |action| self.store.dispatch2(self.selector, action);
                    let on_release = || self.store.dispatchr(Action::Release);

                    match generator {
                        Generator::SimpleWave(config) => {
                            SimpleWaveView::new(&config, dispatch, on_release).ui(ui)
                        }
                        Generator::Noise(_) => todo!(),
                        Generator::SubSynth(config) => {
                            SubSynthView::new(&config, on_release, &self.store, self.selector.0)
                                .ui(ui);
                        }
                    };
                },
            );
    }
}
