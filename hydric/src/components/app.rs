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
use crate::state::{Action, Store};
use egui::Pos2;
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use poll_promise::Promise;
use shared::model::Track;
use shared::types::Beats;

pub struct App {
    pub store: Store,
    pub track_list: Vec<String>,
    pub volume: f32,
    pub audio: Vec<f32>,
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
            store: Store::default(),
            track_list: vec![],
            volume: 1.0,
            audio: vec![],
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
                        ui.text_edit_singleline(&mut self.store.project.name);
                        save_button(self, ui);
                        load_control(self, ui);
                    });
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                            ui.add(
                                egui::Slider::from_get_set(20.0..=200.0, |it| {
                                    it.map(|it| self.store.dispatch(Action::SetBpm(it as Beats)));
                                    self.store.project.bpm.into()
                                })
                                .text("BPM")
                                .logarithmic(true),
                            );
                        });
                        toggle_window_panel(self, ui);
                    });

                    if self.show_envelope {
                        egui::Window::new("Envelope")
                            .default_pos(Pos2 { x: 600.0, y: 125.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                envelope_control(&mut self.store, ui);
                            });
                    }
                    if self.show_generator {
                        egui::Window::new("Generator")
                            .default_pos(Pos2 { x: 1100.0, y: 20.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                generator_control(&mut self.store, ui);
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
                                for i in 0..self.store.project.mixer[0].effects.len() {
                                    ui.separator();
                                    effect_control(&mut self.store, i, ui);
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
