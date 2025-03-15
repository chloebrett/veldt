use std::collections::BTreeSet;

use super::effect_control;
use super::envelope_control;
use super::generator_control;
use super::key_control;
use super::load_control;
use super::notes_control;
use super::play_control;
use super::save_button;
use super::toggle_window_panel;
use crate::audio_player::Handle;
use crate::rpc::load_track_list;
use egui::Pos2;
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use poll_promise::Promise;
use shared::model::PlacedNote;
use shared::model::Track;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, Note, PitchName, Scale,
    ScaleValue, SimpleWaveConfig, WaveType,
};

pub struct App {
    pub track_name: String,
    pub track_list: Vec<String>,
    pub track: Track,
    pub volume: f32,
    pub bpm: f32,
    pub generator: GeneratorInstance,
    pub mixer_channels: Vec<MixerChannel>,
    pub audio: Vec<f32>,
    pub notes: Vec<Note>,
    pub key: ScaleValue,
    pub scale: Scale,
    pub handle: Option<Handle>,
    pub track_list_promise: Promise<Option<Vec<String>>>,
    pub track_promise: Option<Promise<Option<Track>>>,
    pub server_render_promise: Option<Promise<Option<Vec<f32>>>>,
    pub save_track_promise: Option<Promise<Option<()>>>,
    pub show_effects: bool,
    pub show_envelope: bool,
    pub show_generator: bool,
    pub show_scale: bool,
    pub load_track_name: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            track_name: "My Track".to_owned(),
            track_list: vec![],
            track: Track {
                notes: BTreeSet::<PlacedNote>::new()
            },
            volume: 1.0,
            bpm: 120.0,
            generator: GeneratorInstance {
                id: 0,
                kind: GeneratorType::SimpleWave {
                    config: SimpleWaveConfig {
                        wave: WaveType::Sine,
                        envelope: AdsrEnvelope {
                            attack: 0.1,
                            decay: 0.1,
                            sustain: 0.8,
                            release: 0.1,
                        },
                        osc_count: 4,
                        detune_cents: 5.0,
                    },
                },
                meta: GeneratorMeta { volume: 1.0 },
            },
            mixer_channels: vec![MixerChannel {
                effects: vec![
                    EffectInstance {
                        effect: Effect::SimpleEq {
                            config: EqConfig {
                                kind: EqType::SimpleResonator,
                                fc: 1000.0,
                                q: 1.0,
                            },
                        },
                        meta: EffectMeta { id: 0, wet: 1.0 },
                    },
                    EffectInstance {
                        effect: Effect::SimpleDelay {
                            config: DelayConfig {
                                amplitude: 0.5,
                                delay_ms: 250.0,
                            },
                        },
                        meta: EffectMeta { id: 1, wet: 0.5 },
                    },
                ],
            }],
            audio: vec![],
            notes: vec![Note {
                pitch_name: PitchName {
                    scale_value: ScaleValue::A,
                    octave: 4,
                },
                beats: 1.0,
            }],
            key: ScaleValue::A,
            scale: Scale::Chromatic,
            handle: None,
            track_list_promise: Promise::spawn_local(async move { load_track_list().await }),
            track_promise: None,
            server_render_promise: None,
            save_track_promise: None,
            show_effects: false,
            show_envelope: false,
            show_generator: false,
            show_scale: false,
            load_track_name: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    ui.heading("Veldt");
                    ui.horizontal(|ui| {
                        ui.label("Track name: ");
                        ui.text_edit_singleline(&mut self.track_name);
                        save_button(self, ui);
                        load_control(self, ui);
                    });
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                            ui.add(
                                egui::Slider::new(&mut self.bpm, 20.0..=200.0)
                                    .text("BPM")
                                    .logarithmic(true),
                            );
                        });
                        toggle_window_panel(self, ui);
                    });

                    let generator_type: &mut GeneratorType = &mut self.generator.kind;
                    let generator_config: &mut SimpleWaveConfig = match generator_type {
                        GeneratorType::SimpleWave { config } => config,
                    };
                    if self.show_envelope {
                        egui::Window::new("Envelope")
                            .default_pos(Pos2 { x: 600.0, y: 125.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                envelope_control(&mut generator_config.envelope, ui);
                            });
                    }
                    if self.show_generator {
                        egui::Window::new("Generator")
                            .default_pos(Pos2 { x: 1100.0, y: 20.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                generator_control(generator_config, ui);
                            });
                    }
                    if self.show_effects {
                        egui::Window::new("Effects")
                            .default_pos(Pos2 {
                                x: 1100.0,
                                y: 150.0,
                            })
                            .resizable(false)
                            .show(ctx, |ui| {
                                for i in 0..self.mixer_channels[0].effects.len() {
                                    ui.separator();
                                    effect_control(&mut self.mixer_channels[0].effects[i], ui);
                                }
                                ui.separator();
                            });
                    }
                    if self.show_scale {
                        egui::Window::new("Scale")
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                key_control(self, ui);
                            });
                    }
                    ui.separator();
                    notes_control(self, ui, ctx);
                    ui.separator();
                    play_control(self, ui);

                    ui.separator();

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        egui::warn_if_debug_build(ui);
                    });
                });
        });
    }
}
