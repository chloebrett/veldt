use super::{MixerMatrixView, effect_name};
use crate::GetSet;
use crate::components::AudioLevel;
use crate::local_state::LocalState;
use crate::playback::AudioPlayer;
use crate::view::View;
use crate::widget::StateWindow;
use crate::widget::int_slider;
use crate::widget::{add_knob, default_window, styled_knob};
use crate::window_state::WindowKind;
use egui::CornerRadius;
use egui::Shape;
use egui::{Button, Color32, Frame, InnerResponse, Layout, Response, Stroke, Ui, Widget};
use mesic::from_db;
use mesic::to_db;
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{
    Action, EffectSelector, FloatField, IndexField, MixerSelector, MoveField, Store, TypeField,
};
use strum::IntoEnumIterator;

pub struct MixerView<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
    player: &'a AudioPlayer,
}

impl<'a> MixerView<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState, player: &'a AudioPlayer) -> Self {
        Self {
            store,
            local_state,
            player,
        }
    }
}

impl View for MixerView<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            store,
            local_state,
            player,
            ..
        } = self;
        let mixer_sel = local_state
            .active_mixer_channel
            .get()
            .unwrap_or(MixerSelector(0));
        let MixerSelector(mixer_index) = mixer_sel;

        let mixer = &store.select(&mixer_sel);
        let dispatch_mixer = |action| store.dispatch(&mixer_sel, action);
        let on_release = || store.dispatchr(Action::Release);
        let edit_state = local_state.mixer_edit_state.get();

        // Keep track of an object being dragged.
        let mut from_to = None;

        StateWindow(
            default_window("Mixer")
                .id("mixer".into())
                .default_pos(local_state.window_state.get_pos(WindowKind::Mixer)),
        )
        .show_with_closure(
            ui,
            local_state.window_state.get_visible(WindowKind::Mixer),
            |_| {
                local_state
                    .window_state
                    .set_visible(WindowKind::Mixer, false)
            },
            |ui| {
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.add_space(8.0);

                    ui.vertical(|ui| {
                        let matrix = &store.get().project.mixer.matrix;
                        let row_titles: Vec<String> =
                            (0..matrix.channels).map(|i| format!("Ch{i}")).collect();
                        let col_titles: Vec<String> = (0..matrix.channels)
                            .map(|i| {
                                if i == 0 {
                                    "Main in".to_string()
                                } else {
                                    format!("Ch{i} in")
                                }
                            })
                            .collect();
                        MixerMatrixView::new(matrix, row_titles, col_titles, store, on_release)
                            .ui(ui);
                    });
                });

                ui.add_space(8.0);

                // TODO: better UI than a slider for this!
                let max_channel_index = (store.get().project.mixer.channels.len() - 1) as i32;
                int_slider(
                    ui,
                    "Selected channel",
                    mixer_index as f64,
                    |it| {
                        local_state
                            .active_mixer_channel
                            .set(Some(MixerSelector(it as usize)))
                    },
                    0..=max_channel_index,
                    /* on_release= */
                    || {}, // no-op on_release since this doesn't use the store.
                );

                ui.separator();

                ui.horizontal(|ui| {
                    let heading = if mixer_index == 0 {
                        "Main channel"
                    } else {
                        &format!("Channel {mixer_index}")
                    };
                    ui.heading(heading);
                });

                ui.separator();
                ui.horizontal(|ui| {
                    let dispatch_volume =
                        |it| dispatch_mixer(Action::SetFloat(FloatField::Volume, from_db(it)));
                    AudioLevel::new(player, to_db(mixer.volume), dispatch_volume, on_release)
                        .ui(ui);
                    ui.vertical(|ui| {
                        ui.with_layout(Layout::default(), |ui| {
                            // Set background to transparent to avoid a lightened background caused by drag
                            // and drop.
                            ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                            ui.dnd_drop_zone::<EffectLocation, ()>(Frame::default(), |ui| {
                                for effect_index in 0..mixer.effects.len() {
                                    let effect_sel = mixer_sel.downcast_effect(effect_index);
                                    let dispatch_effect =
                                        |action: Action| store.dispatch(&effect_sel, action);
                                    // TODO Determine if this is the best way to do this.
                                    // There seems to be no way to render an object once then pass the
                                    // response into the `dnd_drag_zone` if `edit_state` is true.
                                    let render_effect_widget = |ui: &mut Ui| {
                                        ui.add_enabled(
                                            !edit_state,
                                            EffectWidget::new(
                                                &mixer.effects[effect_index],
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
            },
        );

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
            add_knob(
                ui,
                styled_knob(
                    "Wet",
                    meta.wet,
                    |it| dispatch(Action::SetFloat(FloatField::Wet, it)),
                    0.0..=1.0,
                )
                .with_neutral(0.5),
                on_release,
            );

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
