use super::effect_name;
use crate::WindowState;
use crate::view::WindowView;
use crate::widget::{checkbox, default_window, knob};
use egui::{Context, Pos2};
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{Action, FloatField, IndexField, Selector, Store, TypeField};
use strum::IntoEnumIterator;

pub struct MixerWindow<'a> {
    window_state: &'a mut WindowState,
    store: &'a Store,
}

impl<'a> MixerWindow<'a> {
    pub fn new(window_state: &'a mut WindowState, store: &'a Store) -> Self {
        MixerWindow {
            window_state,
            store,
        }
    }
}

impl WindowView for MixerWindow<'_> {
    fn ui(&mut self, ctx: &Context) {
        let MixerWindow {
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
            .show(ctx, |ui| {
                for effect_index in 0..mixer.effects.len() {
                    let dispatch_effect = |action| {
                        store.dispatch(&Selector::Effect(mixer_index, effect_index), action)
                    };
                    let effect = &mixer.effects[effect_index];
                    ui.label(effect_name(&effect.effect));

                    let show = &mut window_state.effects[mixer_index][effect_index];
                    let text = if *show { "Hide" } else { "Show" };
                    if ui.button(text).clicked() {
                        *show = !*show;
                    }

                    let meta = &effect.meta;
                    knob(
                        ui,
                        "Wet",
                        meta.wet,
                        |it| dispatch_effect(Action::SetFloat(FloatField::Wet, it)),
                        0.0..=1.0,
                        on_release,
                    );
                    checkbox(
                        ui,
                        meta.mute,
                        |it| dispatch_effect(Action::SetChild(TypeField::Mute(it))),
                        "Mute",
                    );
                    if ui.button("Delete").clicked() {
                        dispatch_mixer(Action::DeleteChild(IndexField::Effect(effect_index)));
                        window_state.effects[mixer_index].remove(effect_index);

                        // Skip iterating for this frame.
                        break;
                    }
                    if effect_index > 0 && ui.button("🔼").clicked() {
                        // TODO: rearranging effects like this while their windows are open causes the
                        // windows to reset position - because the IDs change. Should we have stable
                        // IDs instead / as well?
                        dispatch_mixer(Action::MoveEffectUp(effect_index));
                        // TODO: notice how the store state and the window state depend on each other.
                        // How can we sync these up automatically?
                        window_state.effects[mixer_index].swap(effect_index, effect_index - 1);
                    }
                    if effect_index < mixer.effects.len() - 1 && ui.button("🔽").clicked() {
                        dispatch_mixer(Action::MoveEffectDown(effect_index));
                        window_state.effects[mixer_index].swap(effect_index, effect_index + 1);
                    }
                    ui.separator();
                }

                for effect in Effect::iter() {
                    let text = format!("New {}", effect_name(&effect));
                    if ui.button(text).clicked() {
                        let instance = EffectInstance {
                            effect,
                            meta: EffectMeta::default(),
                        };
                        dispatch_mixer(Action::AddChild(TypeField::Effect(instance)));
                        window_state.effects[mixer_index].push(false);
                    }
                }

                ui.label(format!("Mixer channel {}", mixer_index + 1));
            });
    }
}
