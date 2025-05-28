use egui::Ui;
use shared::model::{Effect, EffectInstance, EffectMeta};
use state::{Action, MixerSelector, Store, TypeField};
use strum::IntoEnumIterator;

use crate::{
    components::effect::effect_name,
    local_state::{GetSet, LocalState},
    view::View,
    window_state::WindowKind,
};

pub struct EffectMenuOptions<'a> {
    store: &'a Store,
    local_state: &'a LocalState,
}

impl<'a> EffectMenuOptions<'a> {
    pub fn new(store: &'a Store, local_state: &'a LocalState) -> Self {
        Self { store, local_state }
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
                            self.local_state.active_mixer_channel.set(Some(mixer_sel));
                            self.local_state
                                .window_state
                                .set_visible(WindowKind::Mixer, true);
                        }
                    }
                });
            }
        });
    }
}
