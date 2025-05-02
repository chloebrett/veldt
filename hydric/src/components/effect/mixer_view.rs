use std::sync::Arc;

use super::effect_name;
use crate::WindowState;
use crate::view::View;
use crate::widget::{default_window, knob};
use egui::{Button, Color32, Frame, Pos2, Stroke, Ui};
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{Action, FloatField, IndexField, Store, TypeField};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    row: usize,
}

impl View for MixerView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            window_state,
            store,
            ..
        } = self;
        let mixer_sel = window_state.mixer.channel;
        let mixer = &store.select(&mixer_sel);
        let dispatch_mixer = |action| store.dispatch2(&mixer_sel, action);
        let on_release = || store.dispatchr(Action::Release);
        let mixer_index = mixer_sel.0;

        // Keep track of from where and to an object is dropped.
        let mut from: Option<Arc<Location>> = None;
        let mut to = None;

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
                let frame = Frame::default();
                let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                    for effect_index in 0..mixer.effects.len() {
                        let effect_sel = mixer_sel.downcast_effect(effect_index);
                        let id = egui::Id::new(("effect_config", effect_index));
                        let location = Location { row: effect_index };
                        let response = ui
                            .dnd_drag_source(id, location, |ui| ui.label("Drag"))
                            .response;
                        if let (Some(pointer), Some(hovered_payload)) = (
                            ui.input(|i| i.pointer.interact_pos()),
                            response.dnd_hover_payload::<Location>(),
                        ) {
                            let rect = response.rect;

                            // Preview Insertion
                            let stroke = Stroke::new(1.0, Color32::WHITE);
                            let insert_row_index = if *hovered_payload == location {
                                // Object is dragging onto itself.
                                ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                effect_index
                            } else if pointer.y < rect.center().y {
                                // Object is dragging from above
                                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                effect_index
                            } else {
                                // Object is dragging from below
                                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                effect_index
                            };

                            if let Some(dragged_payload) = response.dnd_release_payload() {
                                // Object was dropped here
                                from = Some(dragged_payload);
                                to = Some(Location { row: effect_index });
                            }
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
                                if effect_index < mixer.effects.len() - 1
                                    && ui.button("🔽").clicked()
                                {
                                    dispatch_mixer(Action::MoveEffectDown(effect_index));
                                }
                            });
                            let dispatch_effect = |action| store.dispatch2(&effect_sel, action);
                            let effect = &mixer.effects[effect_index];

                            let show = window_state.effects.get(effect_sel);
                            let text = effect_name(&effect.it);
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
                                Button::new(text).selected(window_state.effects.get(effect_sel)),
                            );
                            if response.clicked() {
                                window_state.effects.set(effect_sel, !show);
                            }
                        });
                        ui.separator();
                    }
                });

                ui.menu_button("Add new effect", |ui| {
                    for effect in Effect::iter() {
                        let text = format!("{}", effect_name(&effect));
                        if ui.button(text).clicked() {
                            let instance = EffectInstance {
                                it: effect,
                                meta: EffectMeta::default(),
                            };
                            dispatch_mixer(Action::AddChild(TypeField::Effect(instance)));
                        }
                    }
                });
                if let Some(dragged_payload) = dropped_payload {
                    // The object is dropped but not on any item
                    from = Some(dragged_payload);
                    to = Some(Location { row: usize::MAX });
                }
            });
        if let (Some(from), Some(mut to)) = (from, to) {
            if to.row == usize::MAX {
                dispatch_mixer(Action::AddChild(TypeField::Effect(
                    mixer.effects[from.row].clone(),
                )));
                dispatch_mixer(Action::DeleteChild(IndexField::Effect(from.row)));
            }
        }
    }
}
