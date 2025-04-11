use super::effect_name;
use crate::components::WindowState;
use crate::widget::{FloatRange, checkbox, default_window, knob};
use egui::Pos2;
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{Action, Selector, Store};
use strum::IntoEnumIterator;

pub fn mixer_control(ctx: &egui::Context, window_state: &mut WindowState, store: &Store) {
    // The currently active mixer index.
    let mixer_index = window_state.mixer.channel;
    let mixer = &store.get().project.mixer[mixer_index];
    let mixer_sel = Selector::Mixer(mixer_index);

    default_window("Mixer")
        .id(format!("mixer_{mixer_index}").into())
        .default_pos(Pos2 {
            x: 1000.0,
            y: 150.0,
        })
        .open(&mut window_state.mixer.visible)
        .show(ctx, |ui| {
            for effect_index in 0..mixer.effects.len() {
                let sel = Selector::Effect(mixer_index, effect_index);

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
                    |it| store.dispatch(&sel, Action::SetEffectWet(it)),
                    FloatRange(0.0, 1.0),
                );
                checkbox(
                    ui,
                    meta.mute,
                    |it| store.dispatch(&sel, Action::SetEffectMute(it)),
                    "Mute",
                );
                if ui.button("Delete").clicked() {
                    store.dispatch(&mixer_sel, Action::DeleteEffect(effect_index));
                    window_state.effects[mixer_index].remove(effect_index);

                    // Skip iterating for this frame.
                    break;
                }
                if effect_index > 0 && ui.button("🔼").clicked() {
                    // TODO: rearranging effects like this while their windows are open causes the
                    // windows to reset position - because the IDs change. Should we have stable
                    // IDs instead / as well?
                    store.dispatch(&mixer_sel, Action::MoveEffectUp(effect_index));
                    // TODO: notice how the store state and the window state depend on each other.
                    // How can we sync these up automatically?
                    window_state.effects[mixer_index].swap(effect_index, effect_index - 1);
                }
                if effect_index < mixer.effects.len() - 1 && ui.button("🔽").clicked() {
                    store.dispatch(&mixer_sel, Action::MoveEffectDown(effect_index));
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
                    store.dispatch(&mixer_sel, Action::AddEffect(instance));
                    window_state.effects[mixer_index].push(false);
                }
            }

            ui.label(format!("Mixer channel {}", mixer_index + 1));
        });
}
