use egui::Ui;
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{Action, MixerSelector, Store, TypeField};
use strum::IntoEnumIterator;

use crate::{components::effect::effect_name, view::View, window_state::WindowState};

pub struct EffectMenuOptions<'a> {
    store: &'a Store,
    window_state: &'a mut WindowState,
}

impl<'a> EffectMenuOptions<'a> {
    pub fn new(store: &'a Store, window_state: &'a mut WindowState) -> Self {
        Self {
            store,
            window_state,
        }
    }
}

impl View for EffectMenuOptions<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        ui.menu_button("Add new", |ui| {
            for effect in Effect::iter() {
                ui.menu_button(effect_name(&effect.clone()), |ui| {
                    for mixer_index in 0..self.store.get().project.mixer.channels.len() {
                        if ui.button(format!("Mixer {}", mixer_index + 1)).clicked() {
                            let instance = EffectInstance {
                                it: effect.clone(),
                                meta: EffectMeta::default(),
                            };
                            let mixer_sel = MixerSelector(mixer_index);
                            self.store.dispatch(
                                &mixer_sel,
                                Action::AddChild(TypeField::Effect(instance)),
                            );
                            // Open mixer window.
                            self.window_state.mixer.channel = mixer_sel;
                            self.window_state.mixer.visible = true;
                        }
                    }
                });
            }
        });
    }
}

//testing push 