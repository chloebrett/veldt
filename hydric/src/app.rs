use crate::audio_player::{Handle, play};
use crate::envelope_control;
use crate::audio_render::render;
use crate::note_save::{load_note_list, load_notes, save_notes};
use egui::{
    Color32, Rect, ScrollArea, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    scroll_area::ScrollBarVisibility, vec2,
};
use mesic::{SAMPLE_RATE, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, Note, PitchName, Scale, ScaleValue,
    SimpleWaveConfig, WaveType,
};
use strum::IntoEnumIterator;

pub struct App {
    track_name: String,
    track_list: Vec<String>,
    volume: f32,
    bpm: f32,
    envelope: AdsrEnvelope,
    osc_count: u32,
    detune: f32,
    resonant_freq: f32,
    q: f32,
    eq_wet: f32,
    eq_type: EqType,
    wave_type: WaveType,
    audio: Vec<f32>,
    notes: Vec<Note>,
    key: ScaleValue,
    scale: Scale,
    delay_ms: f32,
    delay_wet: f32,
    delay_amplitude: f32,
    handle: Option<Handle>,
    notes_list_promise: Promise<Vec<String>>,
    notes_promise: Option<Promise<Vec<Note>>>,
    server_render_promise: Option<Promise<Vec<f32>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            track_name: "My Track".to_owned(),
            track_list: vec![],
            volume: 1.0,
            bpm: 120.0,
            envelope: AdsrEnvelope {
                attack: 0.1,
                decay: 0.1,
                sustain: 0.8,
                release: 0.1,
            },
            osc_count: 4,
            detune: 5.0,
            resonant_freq: 1000.0,
            q: 1.0,
            eq_wet: 1.0,
            eq_type: EqType::SimpleResonator,
            wave_type: WaveType::Sine,
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
            delay_ms: 250.0,
            delay_wet: 0.5,
            delay_amplitude: 0.5,
            handle: None,
            notes_list_promise: Promise::spawn_local(async move { load_note_list().await }),
            notes_promise: None,
            server_render_promise: None,
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

    fn generator_control(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_label("Wave type")
            .selected_text(self.wave_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.wave_type, WaveType::Sine, "Sine");
                ui.selectable_value(&mut self.wave_type, WaveType::Square, "Square");
                ui.selectable_value(&mut self.wave_type, WaveType::Saw, "Saw");
                ui.selectable_value(&mut self.wave_type, WaveType::Triangle, "Triangle");
            });
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
    }

    fn eq_control(&mut self, ui: &mut Ui) {
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
                    ui.selectable_value(&mut self.eq_type, eq_type.clone(), eq_type.to_string());
                }
            });
        ui.add(egui::Slider::new(&mut self.eq_wet, 0.0..=1.0).text("EQ wet"));
    }

    fn delay_control(&mut self, ui: &mut Ui) {
        ui.add(egui::Slider::new(&mut self.delay_amplitude, 0.0..=1.0).text("Delay amplitude"));
        ui.add(
            egui::Slider::new(&mut self.delay_ms, 1.0..=1000.0)
                .text("Delay ms")
                .logarithmic(true),
        );
        ui.add(egui::Slider::new(&mut self.delay_wet, 0.0..=1.0).text("Delay wet"));
    }

    fn key_control(&mut self, ui: &mut Ui) {
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
    }

    fn notes_control(&mut self, ui: &mut Ui) {
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
    }

    fn save_control(&mut self, ui: &mut Ui) {
        if ui.button("Save").clicked() {
            let save_name = self.track_name.clone();
            let notes_to_save = self.notes.clone();
            let _save_thread =
                Promise::spawn_local(async move { save_notes(save_name, notes_to_save).await });
            self.notes_list_promise = Promise::spawn_local(async move { load_note_list().await });
        }

        if let Some(list) = self.notes_list_promise.ready() {
            self.track_list = list.to_vec()
        }
        egui::ComboBox::from_label("Saved Tracks")
            .selected_text(self.track_name.clone())
            .show_ui(ui, |ui| {
                for name in self.track_list.iter() {
                    ui.selectable_value(&mut self.track_name, name.clone(), name);
                }
            });
        if ui.button("Load").clicked() {
            let load_name = self.track_name.clone();
            self.notes_promise = Some(Promise::spawn_local(
                async move { load_notes(load_name).await },
            ))
        }
        if let Some(notes_promise) = &self.notes_promise {
            if let Some(notes) = notes_promise.ready() {
                self.notes = notes.to_vec()
            }
        }
    }

    fn play_control(&mut self, ui: &mut Ui) {
        if ui.button("Play (local)").clicked() {
            let track = create_track(self.notes.clone());
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
            let generator = GeneratorInstance {
                id: 0,

                kind: GeneratorType::SimpleWave {
                    config: SimpleWaveConfig {
                        wave: self.wave_type,

                        envelope: self.envelope.clone(),

                        osc_count: self.osc_count,

                        detune_cents: self.detune,
                    },
                },

                meta: GeneratorMeta { volume: 1.0 },
            };
            self.audio = local_render(&track, effects, generator, self.bpm)
                .into_iter()
                .map(|sample| sample.clamp(-1.0, 1.0))
                .collect();
            self.handle = Some(play(&self.audio));
        }
        if let Some(render_promise) = &self.server_render_promise {
            if let Some(server_audio) = render_promise.ready() {
                if ui.button("Play (server)").clicked() {
                    self.audio = server_audio
                        .to_vec()
                        .into_iter()
                        .map(|sample| sample.clamp(-1.0, 1.0))
                        .collect::<Vec<f32>>();
                    self.handle = Some(play(&self.audio))
                }
            }
        }
        if ui.button("Load audio (server)").clicked() {
            let track = create_track(self.notes.clone());
            self.server_render_promise =
                Some(Promise::spawn_local(async move { render(track).await }))
        }
        self.audio_vis(ui);
    }

    fn audio_vis(&self, ui: &mut Ui) {
        let audio_len = self.audio.len() as f32;

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let desired_size = vec2(500.0, 100.0);
            let (_id, rect) = ui.allocate_space(desired_size);
            let to_screen = emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=audio_len, 1.0..=-1.0),
                rect,
            );

            let points: Vec<_> = self
                .audio
                .iter()
                .enumerate()
                .map(|(i, sample)| pos2(i as f32, *sample))
                .collect();

            let thickness = 1.0;
            let mut shapes = vec![];
            shapes.push(epaint::Shape::line(
                points.into_iter().map(|it| to_screen * it).collect(),
                PathStroke::new(thickness, Color32::WHITE),
            ));
            if let Some(handle) = &self.handle {
                let current_timestamp = chrono::offset::Utc::now();
                let time_delta_ms: i64 =
                    (current_timestamp - handle.start_timestamp).num_milliseconds();
                let audio_duration_ms: f32 = audio_len / (SAMPLE_RATE as f32) * 1000.0;
                let playthrough_ratio: f32 = (time_delta_ms as f32) / audio_duration_ms;
                let playthrough_samples: f32 = playthrough_ratio * audio_len;

                if (0.0..=1.0).contains(&playthrough_ratio) {
                    let red_line = epaint::Shape::line(
                        vec![
                            to_screen * pos2(playthrough_samples, -1.0),
                            to_screen * pos2(playthrough_samples, 1.0),
                        ],
                        PathStroke::new(thickness, Color32::RED),
                    );
                    shapes.push(red_line);
                }
            }
            ui.painter().extend(shapes);
        });
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
                    });

                    ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                    ui.add(
                        egui::Slider::new(&mut self.bpm, 20.0..=200.0)
                            .text("BPM")
                            .logarithmic(true),
                    );

                    ui.separator();
                    envelope_control(&mut self.envelope, ui);
                    ui.separator();
                    self.generator_control(ui);
                    ui.separator();
                    self.eq_control(ui);
                    ui.separator();
                    self.delay_control(ui);
                    ui.separator();
                    self.key_control(ui);
                    ui.separator();
                    self.notes_control(ui);
                    ui.separator();
                    self.save_control(ui);
                    ui.separator();
                    self.play_control(ui);

                    ui.separator();

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        egui::warn_if_debug_build(ui);
                    });
                });
        });
    }
}
