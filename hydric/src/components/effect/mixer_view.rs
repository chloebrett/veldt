use super::effect_name;
use crate::WindowState;
use crate::view::View;
use crate::widget::{default_window, knob};
use egui::{Button, Pos2, Ui};
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{Action, FloatField, IndexField, Selector, Store, TypeField};
use strum::IntoEnumIterator;

pub struct MixerView<'a> {
    window_state: &'a mut WindowState,
    store: &'a Store,
}

impl<'a> MixerView<'a> {
    pub fn new(window_state: &'a mut WindowState, store: &'a Store) -> Self {
        Self {
            window_state,
            store,
        }
    }
}

impl View for MixerView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            window_state,
            store,
            ..
        } = self;
        let mixer_index = window_state.mixer.channel;
        let mixer = &store.get().project.mixer[mixer_index];
        let dispatch_mixer = |action| store.dispatch(&Selector::Mixer(mixer_index), action);
        let on_release = || store.dispatchr(Action::Release);

        default_window("Mixer")
            .id(format!("mixer_{mixer_index}").into())
            .default_pos(Pos2 {
                x: 1000.0,
                y: 150.0,
            })
            .open(&mut window_state.mixer.visible)
            .show(ui.ctx(), |ui| {
                ui.heading(format!("Mixer channel {}", mixer_index + 1));
                ui.separator();
                for effect_index in 0..mixer.effects.len() {
                    if ui.button("❌").clicked() {
                        dispatch_mixer(Action::DeleteChild(IndexField::Effect(effect_index)));

                        // Skip iterating for this frame.
                        break;
                    }
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            if effect_index > 0 && ui.button("🔼").clicked() {
                                // TODO: rearranging effects like this while their windows are open causes the
                                // windows to reset position - because the IDs change. Should we have stable
                                // IDs instead / as well?
                                dispatch_mixer(Action::MoveEffectUp(effect_index));

                                // TODO: use IDs instead of indexes to refer to effects - otherwise their
                                // visibility is order-dependent.
                            }
                            if effect_index < mixer.effects.len() - 1 && ui.button("🔽").clicked()
                            {
                                dispatch_mixer(Action::MoveEffectDown(effect_index));
                            }
                        });
                        let dispatch_effect = |action| {
                            store.dispatch(&Selector::Effect(mixer_index, effect_index), action)
                        };
                        let effect = &mixer.effects[effect_index];

                        let show = window_state.effects.get((mixer_index, effect_index));
                        let text = effect_name(&effect.effect);
                        let meta = &effect.meta;

                        let mute_response = ui.add(Button::new("Mute").selected(meta.mute));
                        if mute_response.clicked() {
                            dispatch_effect(Action::SetChild(TypeField::Mute(!meta.mute)))
                        }
                        knob(
                            ui,
                            "Wet",
                            meta.wet,
                            |it| dispatch_effect(Action::SetFloat(FloatField::Wet, it)),
                            0.0..=1.0,
                            /* neutral= */ 0.5,
                            on_release,
                        );

                        let response = ui.add(
                            Button::new(text)
                                .selected(window_state.effects.get((mixer_index, effect_index))),
                        );
                        if response.clicked() {
                            window_state.effects.set((mixer_index, effect_index), !show);
                        }
                    });
                    ui.separator();
                }

                ui.menu_button("Add new effect", |ui| {
                    for effect in Effect::iter() {
                        let text = format!("{}", effect_name(&effect));
                        if ui.button(text).clicked() {
                            let instance = EffectInstance {
                                effect,
                                meta: EffectMeta::default(),
                            };
                            dispatch_mixer(Action::AddChild(TypeField::Effect(instance)));
                        }
                    }
                });
            });
    }
}
