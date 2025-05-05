use super::effect_name;
use crate::GetSet;
use crate::WindowState;
use crate::local_state::LocalState;
use crate::view::View;
use crate::widget::int_slider;
use crate::widget::{default_window, knob};
use crate::window_state::WindowStateField;
use egui::CornerRadius;
use egui::Shape;
use egui::{Button, Color32, Frame, InnerResponse, Layout, Pos2, Response, Stroke, Ui, Widget};
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{
    Action, EffectSelector, FloatField, IndexField, MixerSelector, MoveField, Store, TypeField,
};
use strum::IntoEnumIterator;

pub struct MixerView<'a> {
    window_state: &'a mut WindowState,
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> MixerView<'a> {
    pub fn new(
        window_state: &'a mut WindowState,
        store: &'a Store,
        local_state: &'a LocalState,
    ) -> Self {
        Self {
            window_state,
            store,
            local_state,
        }
    }
}

impl View for MixerView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            window_state,
            store,
            local_state,
            ..
        } = self;
        let mixer_sel = window_state.mixer.channel.clone();
        let MixerSelector(mixer_index) = mixer_sel;
        let mixer_sel_mut = &mut window_state.mixer.channel;

        let mixer = &store.select(&mixer_sel);
        let dispatch_mixer = |action| store.dispatch(&mixer_sel, action);
        let on_release = || store.dispatchr(Action::Release);
        let edit_state = local_state.mixer_edit_state.get();

        // Keep track of an object being dragged.
        let mut from_to = None;

        default_window("Mixer")
            .id(format!("mixer").into())
            .default_pos(Pos2 {
                x: 1000.0,
                y: 150.0,
            })
            .open(&mut window_state.mixer.visible)
            .show(ui.ctx(), |ui| {
                ui.heading("Mixer");
                ui.separator();

                // TODO: better UI than a slider for this!
                let max_channel_index = (store.get().project.mixer.len() - 1) as i32;
                int_slider(
                    ui,
                    "Selected channel",
                    mixer_index as f64,
                    |it| *mixer_sel_mut = MixerSelector(it as usize),
                    0..=max_channel_index,
                    /* on_release= */
                    || {}, // no-op on_release since this doesn't use the store.
                );
                ui.separator();

                ui.heading(format!("Mixer channel {}", mixer_index));
                ui.separator();
                ui.with_layout(Layout::default(), |ui| {
                    // Set background to transparent to avoid a lightened background caused by drag
                    // and drop.
                    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                    ui.dnd_drop_zone::<EffectLocation, ()>(Frame::default(), |ui| {
                        for effect_index in 0..mixer.effects.len() {
                            let effect_sel = mixer_sel.downcast_effect(effect_index);
                            let effect_window = &mut window_state.effects;
                            let dispatch_effect =
                                |action: Action| store.dispatch(&effect_sel, action);
                            // TODO Determine if this is the best way to do this.
                            // There seems to be no way to render an object once then pass the
                            // response into the `dnd_drag_zone` if `edit_state` is true.
                            let mut render_effect_widget = |ui: &mut Ui| {
                                ui.add_enabled(
                                    !edit_state,
                                    EffectWidget::new(
                                        &mixer.effects[effect_index],
                                        effect_sel,
                                        effect_window,
                                        dispatch_effect,
                                        on_release,
                                    ),
                                )
                            };
                            if edit_state {
                                let id = egui::Id::new(("effect_config", effect_index));
                                let response = ui
                                    .dnd_drag_source(
                                        id,
                                        EffectLocation::Index(effect_index),
                                        &mut render_effect_widget,
                                    )
                                    .response;
                                // Update `from_to` if an object has been dragged and
                                // released.
                                if let Some(new_from_to) = handle_drag(ui, response, effect_index) {
                                    from_to = Some(new_from_to)
                                };
                            } else {
                                render_effect_widget(ui);
                            }
                        }
                    });
                });
                if edit_state {
                    // Delete drag zone.
                    let response = ui
                        .vertical_centered(|ui| {
                            ui.label("🗑");
                            ui.separator();
                        })
                        .response;
                    if let Some(new_from_to) = handle_delete_drag(ui, response) {
                        from_to = Some(new_from_to)
                    };
                }
                ui.horizontal(|ui| {
                    ui.menu_button("Add new effect", |ui| {
                        for effect in Effect::iter() {
                            let text = effect_name(&effect);
                            if ui.button(text).clicked() {
                                let instance = EffectInstance {
                                    it: effect,
                                    meta: EffectMeta::default(),
                                };
                                dispatch_mixer(Action::AddChild(TypeField::Effect(instance)));
                            }
                        }
                    });
                    // Disable edit state and button if there are no effects.
                    let has_effects = !mixer.effects.is_empty();
                    if !has_effects {
                        local_state.mixer_edit_state.set(false);
                    }
                    if ui
                        .add_enabled(has_effects, Button::new("Edit").selected(edit_state))
                        .clicked()
                    {
                        local_state.mixer_edit_state.set(!edit_state);
                    };
                })
            });

        // Update effects based on drag and drop.
        if let Some((from, to)) = from_to {
            match (from, to) {
                // Object has been deleted.
                (EffectLocation::Index(from_index), EffectLocation::Delete) => {
                    dispatch_mixer(Action::DeleteChild(IndexField::Effect(from_index)))
                }
                // Object has been moved.
                (EffectLocation::Index(from_index), EffectLocation::Index(to_index)) => {
                    dispatch_mixer(Action::MoveChild(MoveField {
                        from_field: IndexField::Effect(from_index),
                        to_field: IndexField::Effect(to_index),
                    }));
                }
                _ => {}
            }
        }
    }
}

/// A widget to display and edit basic effect controls in the MixerView
/// Being a widget that returns a `Response` makes it easier to drag and drop.
struct EffectWidget<'a, F: Fn(Action), G: Fn()> {
    effect: &'a EffectInstance,
    effect_window: &'a mut WindowStateField<EffectSelector>,
    effect_sel: EffectSelector,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EffectWidget<'a, F, G> {
    fn new(
        effect: &'a EffectInstance,
        effect_sel: EffectSelector,
        effect_window: &'a mut WindowStateField<EffectSelector>,
        dispatch: F,
        on_release: G,
    ) -> Self {
        Self {
            effect,
            effect_window,
            effect_sel,
            dispatch,
            on_release,
        }
    }
}

impl<F: Fn(Action), G: Fn()> Widget for EffectWidget<'_, F, G> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            effect,
            effect_window,
            effect_sel,
            dispatch,
            on_release,
        } = self;
        let InnerResponse { response, .. } = ui.horizontal(|ui| {
            let show = effect_window.get(effect_sel);
            let text = effect_name(&effect.it);
            let meta = &effect.meta;

            if ui.add(Button::new("Mute").selected(meta.mute)).clicked() {
                dispatch(Action::SetChild(TypeField::Mute(!meta.mute)))
            }
            knob(
                ui,
                "Wet",
                meta.wet,
                |it| dispatch(Action::SetFloat(FloatField::Wet, it)),
                0.0..=1.0,
                /* neutral= */ 0.5,
                on_release,
            );

            if ui
                .add(Button::new(text).selected(effect_window.get(effect_sel)))
                .clicked()
            {
                effect_window.set(effect_sel, !show)
            }
        });
        ui.separator();
        response
    }
}

/// An enum to store information about a drag and drop zone on the effect mixer.
#[derive(Copy, Clone, Debug)]
enum EffectLocation {
    Index(usize),
    Delete,
}

/// Handle where an object is dragged to and preview where it will be placed.
/// Code adapted from
/// https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/drag_and_drop.rs
fn handle_drag(
    ui: &mut Ui,
    response: Response,
    effect_index: usize,
) -> Option<(EffectLocation, EffectLocation)> {
    let mut from_to = None;
    if let (Some(pointer), Some(hovered_payload)) = (
        ui.input(|i| i.pointer.interact_pos()),
        response.dnd_hover_payload::<EffectLocation>(),
    ) {
        let rect = response.rect;
        // Preview Insertion
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let EffectLocation::Index(index) = *hovered_payload else {
            return None;
        };
        let insert_index = if index == effect_index {
            // Object is dragging onto itself.
            ui.painter().hline(rect.x_range(), rect.center().y, stroke);
            effect_index
        } else if pointer.y < rect.center().y {
            // Object is dragging to above shape.
            ui.painter().hline(rect.x_range(), rect.top(), stroke);
            effect_index
        } else {
            // Object is dragging below shape.
            ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
            effect_index + 1
        };
        if let Some(dragged_payload) = response.dnd_release_payload::<EffectLocation>() {
            // Object was dropped here
            from_to = Some((*dragged_payload, EffectLocation::Index(insert_index)));
        }
    }
    from_to
}

// Handle when an object is dragged into a delete zone.
fn handle_delete_drag(ui: &mut Ui, response: Response) -> Option<(EffectLocation, EffectLocation)> {
    let mut from_to = None;
    if let (Some(..), Some(..)) = (
        ui.input(|i| i.pointer.interact_pos()),
        response.dnd_hover_payload::<EffectLocation>(),
    ) {
        // Preview that an object will be deleted if released.
        let delete_shape = Shape::rect_filled(
            response.rect,
            CornerRadius::same(0),
            Color32::RED.gamma_multiply(0.25),
        );
        ui.painter().add(delete_shape);

        if let Some(dragged_payload) = response.dnd_release_payload::<EffectLocation>() {
            // Object was dropped here
            from_to = Some((*dragged_payload, EffectLocation::Delete));
        }
    }
    from_to
}
