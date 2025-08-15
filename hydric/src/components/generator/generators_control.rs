use super::generator_name;
use crate::components::effect::channel_name;
use crate::local_state::LocalState;
use crate::widget::{StateWindow, add_knob, get_set, selectable_value, styled_knob};
use crate::window_state::WindowKind;
use egui::{Button, ComboBox, Ui};
use shared::model::{GeneratorId, GeneratorInstance};
use state::{Action, FloatField, GeneratorSelector, IndexField, Store, TypeField};

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

                    add_knob(
                        ui,
                        styled_knob(
                            "Volume",
                            meta.volume,
                            |it| store.dispatch(&sel, Action::SetFloat(FloatField::Volume, it)),
                            // TODO: let this go up a bit past 1?
                            0.0..=1.0,
                        )
                        .with_neutral(0.8),
                        on_release,
                    );
                    add_knob(
                        ui,
                        styled_knob(
                            "Pan",
                            meta.pan,
                            |it| store.dispatch(&sel, Action::SetFloat(FloatField::Pan, it)),
                            -1.0..=1.0,
                        )
                        .with_neutral(0.0),
                        on_release,
                    );
                });

                ComboBox::from_id_salt(format!("generator_{:?}_channel", generator_id))
                    .selected_text(channel_name(meta.mixer_channel))
                    .show_ui(ui, |ui| {
                        for channel in 0..store.get().project.mixer.channels.len() {
                            selectable_value(
                                ui,
                                get_set(meta.mixer_channel, |it| {
                                    store.dispatch(&sel, Action::SetIndex(IndexField::Mixer(it)))
                                }),
                                channel,
                                channel_name(channel),
                            );
                        }
                    });

                if index < store.get().project.generators.len() - 1 {
                    ui.separator();
                }
            }

            let mut selected = local_state.new_selected_generator.get();

            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Select one!")
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, "Simple Wave".to_string(), "Simple Wave Generator");
                        ui.selectable_value(&mut selected, "Subtractive Synth".to_string(), "Stringray (Subtrative Synth)");
                        ui.selectable_value(&mut selected, "Noise Generator".to_string(), "Noise Generator");
                    }
                );

                local_state.new_selected_generator.set(selected.clone());

                if ui.button("Add Generator").clicked(){
                    match selected.as_str()  {
                        "Simple Wave" => { 
                            let new_generator = GeneratorInstance {
                                it: Generator::SimpleWave(SimpleWaveConfig {
                                    wave: WaveType::Sine,
                                    envelope: AdsrEnvelope {
                                        attack: 100.0,
                                        decay: 100.0,
                                        sustain: 0.8,
                                        release: 100.0,
                                    },
                                    osc_count: 4,
                                    detune_cents: 5.0,
                                    anti_aliasing_mode: AntiAliasingMode::Off,
                                    oversample_factor: 2,
                                    polyphony_mode: PolyphonyMode::Polyphonic,
                                    polyphony_limit: 2,
                                }),
                                meta: GeneratorMeta {
                                    volume: 1.0,
                                    mute: false,
                                    pan: 0.0,
                                    mixer_channel: 2,
                                }
                            };
                        },
                        "Subtractive Synth" => {
                            let new_generator = GeneratorInstance {
                                it: Generator::Stingray(StingrayConfig::default()),
                                meta: GeneratorMeta {
                                    volume: 1.0,
                                    mute: false,
                                    pan: 0.0,
                                    mixer_channel: 2,
                                }
                            };
                        },
                        "Noise Generator" => {
                            let new_generator = GeneratorInstance {
                                it: Generator::Noise(NoiseConfig::default()),
                                meta: GeneratorMeta {
                                    volume: 1.0,
                                    mute: false,
                                    pan: 0.0,
                                    mixer_channel: 2,
                                }
                            };
                        },
                        &_ => todo!()
                    };
                }
            });
        });
}
