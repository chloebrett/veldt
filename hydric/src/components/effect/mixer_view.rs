use std::sync::Arc;

use super::effect_name;
use crate::WindowState;
use crate::data_state::DataState;
use crate::view::View;
use crate::widget::{default_window, knob};
use crate::window_state::WindowStateField;
use egui::{Button, Color32, Frame, InnerResponse, Pos2, Response, Sense, Stroke, Ui, Widget};
use shared::model::{Effect, EffectInstance, EffectMeta, MixerChannel};
use state::{Action, EffectSelector, FloatField, IndexField, MoveField, Store, TypeField};
use strum::IntoEnumIterator;

struct EffectWidget<'a, F: Fn()> {
    store: &'a Store,
    effect_window: &'a mut WindowStateField<EffectSelector>,
    mixer: &'a MixerChannel,
    effect_index: usize,
    effect_sel: EffectSelector,
    on_release: F,
}

impl<'a, F: Fn()> EffectWidget<'a, F> {
    fn new(
        effect_sel: EffectSelector,
        store: &'a Store,
        effect_window: &'a mut WindowStateField<EffectSelector>,
        mixer: &'a MixerChannel,
        effect_index: usize,
        on_release: F,
    ) -> Self {
        Self {
            store,
            effect_window,
            mixer,
            effect_index,
            effect_sel,
            on_release,
        }
    }
}

impl<G: Fn()> Widget for EffectWidget<'_, G> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            store,
            effect_window,
            mixer,
            effect_index,
            effect_sel,
            on_release,
        } = self;
        let InnerResponse {inner: _, response} = ui.horizontal(|ui| {
            let dispatch_effect = |action| store.dispatch2(&effect_sel, action);
            let effect = &mixer.effects[effect_index];

            let show = effect_window.get(effect_sel);
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

            let response = ui.add(Button::new(text).selected(effect_window.get(effect_sel)));
            if response.clicked() {
                effect_window.set(effect_sel, !show)
            }
        });
        ui.separator();
        response
    }
}

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
        let edit_state = DataState::EditMixerState.get_value(ui).unwrap_or_default();

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
                let edit_response = ui.add(Button::new("Edit").selected(edit_state));
                if edit_response.clicked() {
                    DataState::EditMixerState.set_value(ui, !edit_state)
                };
                ui.separator();
                let frame = Frame::default();
                let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                    for effect_index in 0..mixer.effects.len() {
                        let effect_sel = mixer_sel.downcast_effect(effect_index);
                        let effect_window = &mut window_state.effects;
                        let effect_response = ui.add(EffectWidget::new(
                            effect_sel,
                            store,
                            effect_window,
                            mixer,
                            effect_index,
                            on_release,
                        ));
                        if edit_state {
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
                        }
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
                    to = Some(Location {
                        row: mixer.effects.len() - 1,
                    });
                }
            });
        if let (Some(from), Some(to)) = (from, to) {
            dispatch_mixer(Action::MoveChild(MoveField {
                from_field: IndexField::Effect(from.row),
                to_field: IndexField::Effect(to.row),
            }));
        }
    }
}
