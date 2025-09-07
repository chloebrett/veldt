use super::generator_name;
use crate::components::effect::channel_name;
use crate::local_state::{GetSet, LocalState};
use crate::widget::{StateWindow, add_typable_knob, get_set, selectable_value, styled_knob};
use crate::window_state::WindowKind;
use egui::{Button, ComboBox, Ui};
use shared::model::{Generator, GeneratorId, GeneratorInstance, GeneratorMeta};
use state::{Action, FloatField, GeneratorSelector, IndexField, Store, TypeField};
use strum::IntoEnumIterator;

pub fn generators_control(ui: &mut Ui, local_state: &LocalState, store: &Store) {
    let mut generators: Vec<(&GeneratorId, &GeneratorInstance)> =
        store.get().project.generators.iter().collect();
    generators.sort_by_key(|&(&k, _)| k);

    StateWindow::show_from_window_state(
        ui,
        &local_state.window_state,
        WindowKind::GeneratorList,
        "Generators",
        |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Type: ");
                let curr_generator_type = &local_state.new_generator_type.get();
                ComboBox::from_id_salt("add_generator_combobox")
                    .selected_text(generator_name_from_type(curr_generator_type))
                    .show_ui(ui, |ui| {
                        for generator_type in Generator::iter() {
                            selectable_value(
                                ui,
                                get_set(curr_generator_type.clone(), |new_gen_type| {
                                    local_state.new_generator_type.set(new_gen_type);
                                }),
                                generator_type.clone(),
                                generator_name_from_type(&generator_type),
                            );
                        }
                    });
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Name: ");
                let mut name = local_state.new_generator_name.get();
                let response = ui.text_edit_singleline(&mut name);
                if response.changed() {
                    local_state.new_generator_name.set(name);
                }

                ui.add_space(4.0);

                let add_gen_button = ui.button("Add New Generator");
                if add_gen_button.clicked() {
                    let new_gen = GeneratorInstance {
                        it: local_state.new_generator_type.get(),
                        meta: GeneratorMeta {
                            volume: 1.0,
                            mute: false,
                            pan: 0.0,
                            mixer_channel: 0,
                            name: local_state.new_generator_name.get(),
                        },
                    };
                    store.dispatchr(Action::AddChild(state::TypeField::Generator(new_gen)));
                    local_state.new_generator_name.set("".to_string());
                }
            });

            ui.separator();

            for (index, (generator_id, generator)) in
                store.get().project.generators.iter().enumerate()
            {
                let sel = GeneratorSelector(*generator_id);
                let on_release = || store.dispatchr(Action::Release);

                let label = generator_name(generator);
                let show = local_state
                    .window_state
                    .get_visible(WindowKind::Generator(sel));
                let meta = generator.meta.clone();
                ui.horizontal(|ui| {
                    let mute_response = ui.add(Button::new("Mute").selected(meta.mute));
                    if mute_response.clicked() {
                        store.dispatch(&sel, Action::SetChild(TypeField::Mute(!meta.mute)))
                    }

                    let generator_response = ui.add(Button::new(label).selected(show));
                    if generator_response.clicked() {
                        local_state
                            .window_state
                            .set_visible(WindowKind::Generator(sel), !show);
                    }

                    let volume_knob = styled_knob(
                        meta.volume,
                        |it| store.dispatch(&sel, Action::SetFloat(FloatField::Volume, it)),
                        // TODO: let this go up a bit past 1?
                        0.0..=1.0,
                    )
                    .with_neutral(0.8);
                    let pan_knob = styled_knob(
                        meta.pan,
                        |it| store.dispatch(&sel, Action::SetFloat(FloatField::Pan, it)),
                        -1.0..=1.0,
                    )
                    .with_neutral(0.0);

                    add_typable_knob(
                        ui,
                        volume_knob,
                        "Volume",
                        meta.volume,
                        |it| store.dispatch(&sel, Action::SetFloat(FloatField::Volume, it)),
                        // TODO: let this go up a bit past 1?
                        0.0..=1.0,
                        &on_release,
                    );
                    add_typable_knob(
                        ui,
                        pan_knob,
                        "Pan",
                        meta.pan,
                        |it| store.dispatch(&sel, Action::SetFloat(FloatField::Pan, it)),
                        -1.0..=1.0,
                        &on_release,
                    );
                });
                ui.horizontal(|ui| {
                    ComboBox::from_id_salt(format!("generator_{:?}_channel", generator_id))
                        .selected_text(channel_name(meta.mixer_channel))
                        .show_ui(ui, |ui| {
                            for channel in 0..store.get().project.mixer.channels.len() {
                                selectable_value(
                                    ui,
                                    get_set(meta.mixer_channel, |it| {
                                        store
                                            .dispatch(&sel, Action::SetIndex(IndexField::Mixer(it)))
                                    }),
                                    channel,
                                    channel_name(channel),
                                );
                            }
                        });

                    ui.add_space(4.0);
                    ui.label(generator_name_from_type(&generator.it));

                    let delete_btn = ui.button("Delete");
                    if delete_btn.clicked() {
                        store.dispatchr(Action::DeleteChildById(TypeField::GeneratorId(
                            *generator_id,
                        )));
                    }
                });

                if index < store.get().project.generators.len() - 1 {
                    ui.separator();
                }
            }
        },
    );
}

pub fn generator_name_from_type(generator: &Generator) -> &str {
    match generator {
        Generator::SimpleWave(_) => "Simple Wave Generator",
        Generator::Noise(_) => "Noise Generator",
        Generator::Stingray(_) => "Stingray (Subtractive Synth)",
    }
}
