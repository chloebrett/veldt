use crate::audio_player::AudioPlayer;
use mesic::{SupersawConfig, create_track, render as local_render};
use shared::model::{AdsrEnvelope, Note, PitchName, ScaleValue, WaveType};

pub struct TemplateApp {
    track_name: String,
    volume: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    osc_count: u32,
    detune: f32,
    audio_player: Option<AudioPlayer>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            track_name: "My Track".to_owned(),
            volume: 1.0,
            attack: 0.1,
            decay: 0.1,
            sustain: 0.8,
            release: 0.1,
            osc_count: 4,
            detune: 5.0,
            audio_player: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Veldt");

            ui.horizontal(|ui| {
                ui.label("Track name: ");
                ui.text_edit_singleline(&mut self.track_name);
            });

            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
            ui.add(egui::Slider::new(&mut self.attack, 0.0..=10.0).text("Attack"));
            ui.add(egui::Slider::new(&mut self.decay, 0.0..=10.0).text("Decay"));
            ui.add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain"));
            ui.add(egui::Slider::new(&mut self.release, 0.0..=10.0).text("Release"));
            ui.add(egui::Slider::new(&mut self.osc_count, 0..=24).text("Osc count"));
            ui.add(egui::Slider::new(&mut self.detune, 0.0..=100.0).text("Osc detune"));

            if ui.button("Play").clicked() {
                let envelope = AdsrEnvelope {
                    attack: self.attack,
                    decay: self.decay,
                    sustain: self.sustain,
                    release: self.release,
                };
                let notes = vec![Note {
                    pitch_name: PitchName {
                        scale_value: ScaleValue::A,
                        octave: 4,
                    },
                    beats: 1.0,
                }];
                let track = create_track(notes, WaveType::Saw, 120.0, self.volume, envelope);
                let audio = local_render(
                    &track,
                    SupersawConfig {
                        osc_count: self.osc_count,
                        detune_cents: self.detune,
                    },
                    10000.0,
                    10.0,
                    0.0,
                );
                log::info!("Track {:?}, audio {:?}", track, audio);

                self.audio_player = AudioPlayer::new(&audio).unwrap().into();
            }

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
