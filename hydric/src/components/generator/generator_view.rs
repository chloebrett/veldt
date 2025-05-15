use super::noise::NoiseView;
use super::simple_wave::SimpleWaveView;
use super::stingray::StingrayView;
use crate::view::View;
use crate::widget::StateWindow;
use crate::widget::default_window;
use crate::{LocalState, playback::AudioPlayer};
use egui::{Pos2, Ui};
use shared::model::{Generator, GeneratorInstance};
use state::{Action, GeneratorSelector, Store};

pub struct GeneratorView<'a, F: FnMut()> {
    store: &'a Store,
    selector: &'a GeneratorSelector,
    local_state: &'a LocalState,
    player: &'a mut AudioPlayer,
    visible: bool,
    on_close: F,
}

impl<'a, F: FnMut()> GeneratorView<'a, F> {
    pub fn new(
        store: &'a Store,
        selector: &'a GeneratorSelector,
        local_state: &'a LocalState,
        player: &'a mut AudioPlayer,
        visible: bool,
        on_close: F,
    ) -> Self {
        Self {
            store,
            selector,
            local_state,
            player,
            visible,
            on_close,
        }
    }
}

impl<F: FnMut()> View for GeneratorView<'_, F> {
    fn ui(&mut self, ui: &mut Ui) {
        let instance = &self.store.select(self.selector);

        let title = generator_name(instance);
        StateWindow(default_window(title).default_pos(Pos2 { x: 1100.0, y: 20.0 }))
            .show_with_closure(
                ui,
                self.visible,
                |_| (self.on_close)(),
                |ui| {
                    let generator = instance.it.clone();
                    let dispatch = |action| self.store.dispatch(self.selector, action);
                    let on_release = || self.store.dispatchr(Action::Release);

                    match generator {
                        Generator::SimpleWave(config) => SimpleWaveView::new(
                            *self.selector,
                            &config,
                            self.player,
                            dispatch,
                            on_release,
                        )
                        .ui(ui),
                        Generator::Noise(config) => NoiseView::new(
                            *self.selector,
                            &config,
                            self.player,
                            dispatch,
                            on_release,
                        )
                        .ui(ui),
                        Generator::Stingray(config) => {
                            StingrayView::new(
                                &config,
                                on_release,
                                self.store,
                                self.local_state,
                                self.selector,
                                self.player,
                            )
                            .ui(ui);
                        }
                    };
                },
            );
    }
}

pub fn generator_name(instance: &GeneratorInstance) -> &str {
    match &instance.it {
        Generator::SimpleWave(_) => "Simple Wave Generator",
        Generator::Noise(_) => "Noise Generator",
        Generator::Stingray(_) => "Stingray (Subtractive Synth)",
    }
}
