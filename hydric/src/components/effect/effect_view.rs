use super::{CompressorView, DelayView, EqView, ModDelayView};
use crate::local_state::LocalState;
use crate::view::View;
use crate::widget::{StateWindow, default_window};
use crate::window_state::WindowKind;
use egui::{Pos2, Ui};
use shared::model::Effect;
use state::{Action, EffectSelector, Store};

pub struct EffectView<'a, F: Fn(Action), G: Fn()> {
    local_state: &'a LocalState,
    effect: &'a Effect,
    selector: EffectSelector,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EffectView<'a, F, G> {
    pub fn new(
        store: &'a Store,
        selector: &'a EffectSelector,
        local_state: &'a LocalState,
        dispatch: F,
        on_release: G,
    ) -> Option<Self> {
        let effect = store.try_select(selector)?;
        let effect: &'a Effect = &effect.it;

        Some(Self {
            local_state,
            effect,
            selector: *selector,
            dispatch,
            on_release,
        })
    }
}

impl<F: Fn(Action), G: Fn()> View for EffectView<'_, F, G> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            effect,
            local_state,
            ..
        } = self;
        let dispatch = &self.dispatch;
        let on_release = &self.on_release;
        let title = effect_name(effect);
        let EffectSelector(mixer_index, effect_index) = self.selector;

        StateWindow(
            default_window(title)
                .id(format!("effects_{}_{}", mixer_index, effect_index).into())
                .default_pos(Pos2 {
                    x: 1000.0 + 50.0 * effect_index as f32,
                    y: 150.0 + 50.0 * effect_index as f32,
                }),
        )
        .show_with_closure(
            ui,
            local_state
                .window_state
                .get_visible(WindowKind::Effect(self.selector)),
            |_| {
                local_state
                    .window_state
                    .set_visible(WindowKind::Effect(self.selector), false)
            },
            |ui| {
                match effect {
                    Effect::SimpleEq(config) => EqView::new(config, dispatch, on_release).ui(ui),
                    Effect::Delay(config) => DelayView::new(config, dispatch, on_release).ui(ui),
                    Effect::Compressor(config) => {
                        CompressorView::new(config, dispatch, on_release).ui(ui)
                    }
                    Effect::ModDelay(config) => {
                        ModDelayView::new(config, dispatch, on_release).ui(ui)
                    }
                }

                ui.separator();
                ui.label(format!(
                    "Mixer {} | Effect {}",
                    mixer_index + 1,
                    effect_index + 1
                ));
            },
        );
    }
}

pub fn effect_name(effect: &Effect) -> &str {
    match effect {
        Effect::SimpleEq(_) => "EQ",
        Effect::Delay(_) => "Delay",
        Effect::Compressor(_) => "Compressor",
        Effect::ModDelay(_) => "Modulated Delay",
    }
}
