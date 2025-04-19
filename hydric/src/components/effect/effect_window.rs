use super::{CompressorView, DelayView, EqView, ModDelayView};
use crate::components::WindowState;
use crate::view::{View, WindowView};
use crate::widget::default_window;
use egui::{Context, Pos2};
use shared::model::Effect;
use state::{Action, Store};

pub struct EffectWindow<'a, F: Fn(Action), G: Fn()> {
    visible: &'a mut bool,
    effect: &'a Effect,
    mixer_index: usize,
    effect_index: usize,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EffectWindow<'a, F, G> {
    pub fn new(
        store: &'a Store,
        mixer_index: usize,
        effect_index: usize,
        window_state: &'a mut WindowState,
        dispatch: F,
        on_release: G,
    ) -> Self {
        let effect = &store.get().project.mixer[mixer_index].effects[effect_index].effect;
        let visible = &mut window_state.effects[mixer_index][effect_index];

        EffectWindow {
            visible,
            effect,
            mixer_index,
            effect_index,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> WindowView for EffectWindow<'_, F, G> {
    fn ui(&mut self, ctx: &Context) {
        let EffectWindow {
            visible,
            effect,
            mixer_index,
            effect_index,
            ..
        } = self;
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let title = effect_name(&effect);

        default_window(title)
            .id(format!("effects_{}_{}", mixer_index, effect_index).into())
            .default_pos(Pos2 {
                x: 1000.0 + 50.0 * *effect_index as f32,
                y: 150.0 + 50.0 * *effect_index as f32,
            })
            .open(visible)
            .show(ctx, |ui| {
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
