use crate::{audio_player::AudioPlayer, note_save::save_notes};
use mesic::{SupersawConfig, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType, Note,
    PitchName, Scale, ScaleValue, WaveType,
};
use strum::IntoEnumIterator;

pub struct App {
    track_name: String,
    volume: f32,
    bpm: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    osc_count: u32,
    detune: f32,
    resonant_freq: f32,
    q: f32,
    eq_wet: f32,
    eq_type: EqType,
    wave_type: WaveType,
    audio_player: Option<AudioPlayer>,
    notes: Vec<Note>,
    key: ScaleValue,
    scale: Scale,
    delay_ms: f32,
    delay_wet: f32,
    delay_amplitude: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            track_name: "My Track".to_owned(),
            volume: 1.0,
            bpm: 120.0,
            attack: 0.1,
            decay: 0.1,
            sustain: 0.8,
            release: 0.1,
            osc_count: 4,
            detune: 5.0,
            resonant_freq: 1000.0,
            q: 1.0,
            eq_wet: 1.0,
            eq_type: EqType::SimpleResonator,
            wave_type: WaveType::Sine,
            audio_player: None,
            notes: vec![Note {
                pitch_name: PitchName {
                    scale_value: ScaleValue::A,
                    octave: 4,
                },
                beats: 1.0,
            }],
            key: ScaleValue::A,
            scale: Scale::Chromatic,
            delay_ms: 250.0,
            delay_wet: 0.5,
            delay_amplitude: 0.5,
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
            ui.heading("Veldt");

            ui.horizontal(|ui| {
                ui.label("Track name: ");
                ui.text_edit_singleline(&mut self.track_name);
            });

            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
            ui.add(
                egui::Slider::new(&mut self.bpm, 20.0..=200.0)
                    .text("BPM")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.attack, 0.0..=10.0)
                    .text("Attack")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.decay, 0.0..=10.0)
                    .text("Decay")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.sustain, 0.0..=1.0)
                    .text("Sustain")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.release, 0.0..=10.0)
                    .text("Release")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.osc_count, 1..=24)
                    .text("Osc count")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.detune, 0.0..=100.0)
                    .text("Osc detune")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.resonant_freq, 20.0..=20000.0)
                    .text("Resonant frequency")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.q, 0.1..=100.0)
                    .text("Q value")
                    .logarithmic(true),
            );
            egui::ComboBox::from_label("EQ type")
                .selected_text(format!("{}", self.eq_type))
                .show_ui(ui, |ui| {
                    for eq_type in EqType::iter() {
                        ui.selectable_value(
                            &mut self.eq_type,
                            eq_type.clone(),
                            eq_type.to_string(),
                        );
                    }
                });
            ui.add(egui::Slider::new(&mut self.eq_wet, 0.0..=1.0).text("EQ wet"));
            ui.add(egui::Slider::new(&mut self.delay_amplitude, 0.0..=1.0).text("Delay amplitude"));
            ui.add(
                egui::Slider::new(&mut self.delay_ms, 1.0..=1000.0)
                    .text("Delay ms")
                    .logarithmic(true),
            );
            ui.add(egui::Slider::new(&mut self.delay_wet, 0.0..=1.0).text("Delay wet"));
            egui::ComboBox::from_label("Key")
                .selected_text(self.key.to_string())
                .show_ui(ui, |ui| {
                    for scale_note in ScaleValue::iter() {
                        ui.selectable_value(&mut self.key, scale_note, scale_note.to_string());
                    }
                });
            egui::ComboBox::from_label("Scale")
                .selected_text(self.scale.to_string())
                .show_ui(ui, |ui| {
                    for scale in Scale::iter() {
                        ui.selectable_value(&mut self.scale, scale, scale.to_string());
                    }
                });
            egui::ComboBox::from_label("Wave type")
                .selected_text(self.wave_type.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.wave_type, WaveType::Sine, "Sine");
                    ui.selectable_value(&mut self.wave_type, WaveType::Square, "Square");
                    ui.selectable_value(&mut self.wave_type, WaveType::Saw, "Saw");
                    ui.selectable_value(&mut self.wave_type, WaveType::Triangle, "Triangle");
                });

            let scale_options = create_scale_values(self.scale, self.key);
            for i in 0..self.notes.len() {
                let note = &mut self.notes[i];
                let scale_value = &mut note.pitch_name.scale_value;
                egui::ComboBox::from_id_salt(i)
                    .selected_text(scale_value.to_string())
                    .show_ui(ui, |ui| {
                        for scale_note in scale_options.iter() {
                            ui.selectable_value(scale_value, *scale_note, scale_note.to_string());
                        }
                    });
                ui.add(egui::Slider::new(&mut note.pitch_name.octave, 0..=8).text("Octave"));
                if ui.button("Delete").clicked() {
                    self.notes.remove(i);
                }
            }
            if ui.button("New note").clicked() {
                self.notes.push(Note {
                    pitch_name: PitchName {
                        scale_value: ScaleValue::A,
                        octave: 4,
                    },
                    beats: 1.0,
                });
            }
            let mut save_status = "Save";
            if ui.button(save_status).clicked() {
                let save_name = self.track_name.clone();
                let notes_to_save = self.notes.clone();
                let save_thread =
                    Promise::spawn_local(async move { save_notes(save_name, notes_to_save).await });
            }

            if ui.button("Play (local)").clicked() {
                let envelope = AdsrEnvelope {
                    attack: self.attack,
                    decay: self.decay,
                    sustain: self.sustain,
                    release: self.release,
                };
                let track = create_track(
                    self.notes.clone(),
                    self.wave_type,
                    self.bpm,
                    self.volume,
                    envelope,
                );
                let delay = EffectInstance {
                    effect: Effect::SimpleDelay {
                        config: DelayConfig {
                            amplitude: self.delay_amplitude,

                            delay_ms: self.delay_ms,
                        },
                    },
                    meta: EffectMeta {
                        id: 0,
                        wet: self.delay_wet,
                    },
                };
                let eq = EffectInstance {
                    effect: Effect::SimpleEq {
                        config: EqConfig {
                            kind: self.eq_type.clone(),
                            fc: self.resonant_freq,
                            q: self.q,
                        },
                    },
                    meta: EffectMeta {
                        id: 1,
                        wet: self.eq_wet,
                    },
                };
                let effects = vec![delay, eq];
                let audio = local_render(
                    &track,
                    SupersawConfig {
                        osc_count: self.osc_count,
                        detune_cents: self.detune,
                    },
                    effects,
                );

                self.audio_player = AudioPlayer::new(&audio).unwrap().into();
            }

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
