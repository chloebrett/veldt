use super::{CompressorView, DelayView, EqView, ModDelayView};
use crate::WindowState;
use crate::view::View;
use crate::widget::{StateWindow, default_window};
use egui::{Pos2, Ui};
use shared::model::Effect;
use state::{Action, Store};

pub struct EffectView<'a, F: Fn(Action), G: Fn()> {
    visible: bool,
    on_close: Box<dyn FnMut() + 'a>,
    effect: &'a Effect,
    mixer_index: usize,
    effect_index: usize,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EffectView<'a, F, G> {
    pub fn new(
        store: &'a Store,
        mixer_index: usize,
        effect_index: usize,
        window_state: &'a mut WindowState,
        dispatch: F,
        on_release: G,
    ) -> Option<Self> {
        let effect = &store
            .get()
            .project
            .mixer
            .get(mixer_index)?
            .effects
            .get(effect_index)?
            .effect;
        let visible = window_state.effects.get((mixer_index, effect_index));
        let on_close =
            Box::new(move || window_state.effects.set((mixer_index, effect_index), false));

        Some(EffectView {
            visible,
            on_close,
            effect,
            mixer_index,
            effect_index,
            dispatch,
            on_release,
        })
    }
}

impl<F: Fn(Action), G: Fn()> View for EffectView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let EffectView {
            visible,
            on_close,
            effect,
            mixer_index,
            effect_index,
            ..
        } = self;
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;

        let title = effect_name(effect);

        StateWindow(
            default_window(title)
                .id(format!("effects_{}_{}", mixer_index, effect_index).into())
                .default_pos(Pos2 {
                    x: 1000.0 + 50.0 * *effect_index as f32,
                    y: 150.0 + 50.0 * *effect_index as f32,
                }),
        )
        .show_with_closure(
            ui,
            *visible,
            |_| on_close(),
            |ui| {
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
            },
        );
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
