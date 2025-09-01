use shared::model::{Effect, EffectId, EffectInstance, EffectMeta};
use state::{
    Action, EffectSelector, FloatField, IndexField, MixerSelector, MoveField, Store, TypeField,
};
use strum::IntoEnumIterator;

use crate::{
    local_state::{GetSet, LocalState},
    view::View,
    widget::{StateWindow, styled_knob, add_typable_knob},
    window_state::WindowKind,
};
use egui::{
    Button, Color32, CornerRadius, Frame, InnerResponse, Layout, Response, Shape, Stroke, Ui,
    Widget,
};

use super::{channel_name, effect_name};

pub struct ChannelEffectView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> ChannelEffectView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
    }
}

impl View for ChannelEffectView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            store, local_state, ..
        } = *self;
        let selector = local_state
            .active_mixer_channel
            .get()
            .unwrap_or(MixerSelector(0));
        let mixer = &store.select(&selector);
        let dispatch_mixer = |action| store.dispatch(&selector, action);
        let on_release = || store.dispatchr(Action::Release);
        let edit_state = local_state.mixer_edit_state.get();
        // Keep track of an object being dragged.
        let mut from_to = None;
        let MixerSelector(channel_index) = selector;
        let title = format!("{} effects", channel_name(channel_index));
        StateWindow::show_from_window_state(
            ui,
            &local_state.window_state,
            WindowKind::ChannelEffect,
            &title,
            |ui| {
                ui.vertical(|ui| {
                    ui.with_layout(Layout::default(), |ui| {
                        // Set background to transparent to avoid a lightened background caused by drag
                        // and drop.
                        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                        ui.dnd_drop_zone::<EffectLocation, ()>(Frame::default(), |ui| {
                            for (effect_index, effect_id) in mixer.effect_ids.iter().enumerate() {
                                let effect_sel = EffectSelector(*effect_id);
                                let dispatch_effect =
                                    |action: Action| store.dispatch(&effect_sel, action);
                                // TODO Determine if this is the best way to do this.
                                // There seems to be no way to render an object once then pass the
                                // response into the `dnd_drag_zone` if `edit_state` is true.
                                let render_effect_widget = |ui: &mut Ui| {
                                    ui.add_enabled(
                                        !edit_state,
                                        EffectWidget::new(
                                            store.select(&effect_sel),
                                            effect_sel,
                                            local_state,
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
                                            render_effect_widget,
                                        )
                                        .response;
                                    // Update `from_to` if an object has been dragged and
                                    // released.
                                    if let Some(new_from_to) =
                                        handle_drag(ui, response, effect_index)
                                    {
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
                                    store.dispatchr(Action::AddChild(TypeField::Effect(instance)));
                                    // Hack: use the same logic as the receiver to work out
                                    // what the ID of the just-added effect was (or more
                                    // accurately, will be in the next frame, since dispatches are lazy).
                                    let next_id = *store
                                        .get()
                                        .project
                                        .effects
                                        .clone()
                                        .into_keys()
                                        .max()
                                        .unwrap_or(EffectId(0))
                                        + 1;
                                    let next_id = EffectId(next_id);
                                    let next_index = mixer.effect_ids.len();
                                    dispatch_mixer(Action::AddChildAtIndex(
                                        TypeField::EffectId(next_id),
                                        IndexField::EffectId(next_index),
                                    ));
                                }
                            }
                        });
                        // Disable edit state and button if there are no effects.
                        let has_effects = !mixer.effect_ids.is_empty();
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
                            let effect_id = mixer.effect_ids[from_index];
                            dispatch_mixer(Action::DeleteChildById(TypeField::EffectId(effect_id)))
                        }
                        // Object has been moved.
                        (EffectLocation::Index(from_index), EffectLocation::Index(to_index)) => {
                            dispatch_mixer(Action::MoveChild(MoveField {
                                from_field: IndexField::EffectId(from_index),
                                to_field: IndexField::EffectId(to_index),
                            }));
                        }
                        _ => {}
                    }
                }
            },
        );
    }
}

/// A widget to display and edit basic effect controls in the MixerView
/// Being a widget that returns a `Response` makes it easier to drag and drop.
struct EffectWidget<'a, F: Fn(Action), G: Fn()> {
    effect: &'a EffectInstance,
    local_state: &'a LocalState,
    effect_sel: EffectSelector,
    dispatch: F,
    on_release: G,
}

impl<'a, F: Fn(Action), G: Fn()> EffectWidget<'a, F, G> {
    fn new(
        effect: &'a EffectInstance,
        effect_sel: EffectSelector,
        local_state: &'a LocalState,
        dispatch: F,
        on_release: G,
    ) -> Self {
        Self {
            effect,
            local_state,
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
            local_state,
            effect_sel,
            dispatch,
            on_release,
        } = self;
        let InnerResponse { response, .. } = ui.horizontal(|ui| {
            let show = local_state
                .window_state
                .get_visible(WindowKind::Effect(effect_sel));
            let text = effect_name(&effect.it);
            let meta = &effect.meta;

            if ui.add(Button::new("Mute").selected(meta.mute)).clicked() {
                dispatch(Action::SetChild(TypeField::Mute(!meta.mute)))
            }

            let wet_knob = styled_knob(
                meta.wet,
                |it| dispatch(Action::SetFloat(FloatField::Wet, it)),
                0.0..=1.0,
            ).with_neutral(0.5);
            add_typable_knob(ui, wet_knob, "Wet", meta.wet, |it| dispatch(Action::SetFloat(FloatField::Wet, it)), 0.0..=1.0, &on_release);

            if ui.add(Button::new(text).selected(show)).clicked() {
                local_state
                    .window_state
                    .set_visible(WindowKind::Effect(effect_sel), !show)
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
            ui.painter().add(Shape::rect_filled(
                rect,
                CornerRadius::ZERO,
                Color32::WHITE.gamma_multiply(0.25),
            ));
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
