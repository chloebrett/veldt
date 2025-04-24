use super::{CompressorView, DelayView, EqView, ModDelayView};
use crate::WindowState;
use crate::view::View;
use crate::widget::default_window;
use egui::{Pos2, Ui};
use shared::model::Effect;
use state::{Action, Selector, Store};

pub struct EffectView<'a, F: Fn(Action), G: Fn()> {
    visible: &'a mut bool,
    effect: &'a Effect,
    mixer_index: usize,
    effect_index: usize,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EffectView<'a, F, G> {
    pub fn new(
        store: &'a Store,
        selector: &Selector,
        window_state: &'a mut WindowState,
        dispatch: F,
        on_release: G,
    ) -> Self {
        let Selector::Effect(mixer_index, effect_index) = *selector else {
            panic!()
        };
        let effect = &store.get().project.mixer[mixer_index].effects[effect_index].effect;
        let visible = &mut window_state.effects[mixer_index][effect_index];

        EffectView {
            visible,
            effect,
            mixer_index,
            effect_index,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> View for EffectView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let EffectView {
            visible,
            effect,
            mixer_index,
            effect_index,
            ..
        } = self;
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let title = effect_name(effect);

        default_window(title)
            .id(format!("effects_{}_{}", mixer_index, effect_index).into())
            .default_pos(Pos2 {
                x: 1000.0 + 50.0 * *effect_index as f32,
                y: 150.0 + 50.0 * *effect_index as f32,
            })
            .open(visible)
            .show(ui.ctx(), |ui| {
                match effect {
                    Effect::SimpleEq { config } => EqView::new(config, dispatch, on_release).ui(ui),
                    Effect::SimpleDelay { config } => {
                        DelayView::new(config, dispatch, on_release).ui(ui)
                    }
                    Effect::SimpleCompressor { config } => {
                        CompressorView::new(config, dispatch, on_release).ui(ui)
                    }
                    Effect::ModDelay { config } => {
                        ModDelayView::new(config, dispatch, on_release).ui(ui)
                    }
                }

                ui.separator();
                ui.label(format!(
                    "Mixer {} | Effect {}",
                    *mixer_index + 1,
                    *effect_index + 1
                ));
            });
    }
}

pub fn effect_name(effect: &Effect) -> &str {
    match effect {
        Effect::SimpleEq { .. } => "EQ",
        Effect::SimpleDelay { .. } => "Delay",
        Effect::SimpleCompressor { .. } => "Compressor",
        Effect::ModDelay { .. } => "Modulated Delay",
    }
}
