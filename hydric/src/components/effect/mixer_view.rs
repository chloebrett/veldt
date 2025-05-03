use super::effect_name;
use crate::GetSet;
use crate::WindowState;
use crate::local_state::LocalState;
use crate::view::View;
use crate::widget::{default_window, knob};
use crate::window_state::WindowStateField;
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
        let mixer_sel = window_state.mixer.channel;
        let mixer = &store.select(&mixer_sel);
        let dispatch_mixer = |action| store.dispatch(&mixer_sel, action);
        let on_release = || store.dispatchr(Action::Release);
        let MixerSelector(mixer_index) = mixer_sel;
        let edit_state = local_state.mixer_edit_state.get();

        // Keep track of an object being dragged.
        let mut from = None;
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
                if ui.add(Button::new("Edit").selected(edit_state)).clicked() {
                    local_state.mixer_edit_state.set(!edit_state);
                };
                ui.separator();
                ui.with_layout(Layout::default(), |ui| {
                    // Set background to transparent to avoid a lightened background caused by drag
                    // and drop.
                    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                    ui.dnd_drop_zone::<usize, ()>(Frame::default(), |ui| {
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
                                    .dnd_drag_source(id, effect_index, |ui| {
                                        render_effect_widget(ui)
                                    })
                                    .response;
                                // Update `To` and `From` if an object has been dragged and
                                // released.
                                if let Some((new_from, new_to)) =
                                    handle_drag(ui, response, effect_index)
                                {
                                    from = Some(new_from);
                                    to = Some(new_to);
                                };
                            } else {
                                render_effect_widget(ui);
                            }
                        }
                    });
                });
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
            });
        // Update effects based on drag and drop.
        if let (Some(from), Some(to)) = (from, to) {
            dispatch_mixer(Action::MoveChild(MoveField {
                from_field: IndexField::Effect(from),
                to_field: IndexField::Effect(to),
            }));
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

/// Handle where an object is dragged to and preview where it will be placed.
/// Code adapted from
/// https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/drag_and_drop.rs
fn handle_drag(ui: &mut Ui, response: Response, effect_index: usize) -> Option<(usize, usize)> {
    let mut from_to = None;
    if let (Some(pointer), Some(hovered_payload)) = (
        ui.input(|i| i.pointer.interact_pos()),
        response.dnd_hover_payload::<usize>(),
    ) {
        let rect = response.rect;
        // Preview Insertion
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let insert_index = if *hovered_payload == effect_index {
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
        if let Some(dragged_payload) = response.dnd_release_payload::<usize>() {
            // Object was dropped here
            from_to = Some((*dragged_payload, insert_index));
        }
    }
    from_to
}
