use super::delay_control::delay_control;
use super::envelope_control::envelope_control;
use super::eq_control::eq_control;
use super::generator_control::generator_control;
use super::key_control::key_control;
use super::notes_control::notes_control;
use super::play_control::play_control;
use super::save_control::save_control;
use crate::audio_player::Handle;
use crate::rpc::load_note_list;
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, EqType, GeneratorInstance, GeneratorMeta, GeneratorType, Note, PitchName, Scale,
    ScaleValue, SimpleWaveConfig, WaveType,
};

pub struct App {
    pub track_name: String,
    pub track_list: Vec<String>,
    pub volume: f32,
    pub bpm: f32,
    pub generator: GeneratorInstance,
    pub resonant_freq: f32,
    pub q: f32,
    pub eq_wet: f32,
    pub eq_type: EqType,
    pub audio: Vec<f32>,
    pub notes: Vec<Note>,
    pub key: ScaleValue,
    pub scale: Scale,
    pub delay_ms: f32,
    pub delay_wet: f32,
    pub delay_amplitude: f32,
    pub handle: Option<Handle>,
    pub notes_list_promise: Promise<Option<Vec<String>>>,
    pub notes_promise: Option<Promise<Option<Vec<Note>>>>,
    pub server_render_promise: Option<Promise<Option<Vec<f32>>>>,
    pub save_notes_promise: Option<Promise<Option<()>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            track_name: "My Track".to_owned(),
            track_list: vec![],
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
            resonant_freq: 1000.0,
            q: 1.0,
            eq_wet: 1.0,
            eq_type: EqType::SimpleResonator,
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
            save_notes_promise: None,
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
                    });

                    ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                    ui.add(
                        egui::Slider::new(&mut self.bpm, 20.0..=200.0)
                            .text("BPM")
                            .logarithmic(true),
                    );

                    let generator_type: &mut GeneratorType = &mut self.generator.kind;
                    let generator_config: &mut SimpleWaveConfig = match generator_type {
                        GeneratorType::SimpleWave { config } => config,
                    };

                    ui.separator();
                    envelope_control(&mut generator_config.envelope, ui);
                    ui.separator();
                    generator_control(generator_config, ui);
                    ui.separator();
                    eq_control(self, ui);
                    ui.separator();
                    delay_control(self, ui);
                    ui.separator();
                    key_control(self, ui);
                    ui.separator();
                    notes_control(self, ui);
                    ui.separator();
                    save_control(self, ui);
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
