use super::{
    effect::effect_control,
    generator::{envelope_control, generator_control},
    key_control, load_control,
    note_roll::note_roll_display,
    play::{play_control, sample_control},
    save_button, toggle_window_panel, track_control, track_placement_control, undo_redo_control,
};
use crate::audio_player::Handle;
use crate::promise::AsyncResult;
use crate::widget::{FloatRange, knob, string_observer};
use egui::Pos2;
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use shared::model::{Project, Sample};
use shared::types::Beats;
use state::{Action, Store, get_set};

/// Container for the various promises launchable by the app.
#[derive(Default)]
pub struct AsyncState {
    pub server_render: AsyncResult<Vec<f32>, ()>,
    pub save_project: AsyncResult<(), ()>,
    pub project_list: AsyncResult<Vec<String>, ()>,
    pub load_project: AsyncResult<Project, ()>,
    pub load_sample: AsyncResult<Sample, ()>,
}

#[derive(Default)]
pub struct AudioState {
    pub audio: Vec<f32>,
    pub handle: Option<Handle>,
    pub pre_render: bool,
}

/// Which windows are currently shown.
#[derive(Default)]
pub struct WindowState {
    pub effects: bool,
    pub manual_notes: bool,
    pub generator: bool,
    pub scale: bool,
    pub note_roll: bool,
}

#[derive(Default)]
pub struct App {
    pub store: Store,
    pub async_state: AsyncState,
    pub audio_state: AudioState,
    pub window_state: WindowState,
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
        // Snapshot the state at the start of each frame.
        // TODO: profile this with the FPS counter, since it's a clone.
        // Another option: apply all the actions once per frame, instead of cloning the whole
        // state. Then, the store doesn't need to be mutated the rest of the time, and we don't
        // need to ever clone it.
        self.store.snapshot();

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    ui.heading("Veldt");
                    ui.horizontal(|ui| {
                        let project_name = self.store.get().project.name.clone();
                        let mut name_observer = string_observer(
                            get_set(project_name.clone(), |it| {
                                self.store.dispatchr(Action::SetProjectName(it))
                            }),
                            project_name.clone(),
                        );
                        ui.text_edit_singleline(&mut name_observer);
                        save_button(&self.store, &mut self.async_state, ui);
                        load_control(&self.store, &mut self.async_state, ui);
                    });
                    toggle_window_panel(&mut self.window_state, ui);

                    if self.window_state.generator {
                        let generators = &self.store.get().project.generators;
                        for generator_index in 0..generators.len() {
                            egui::Window::new("Generator")
                                .open(&mut self.window_state.generator)
                                .default_pos(Pos2 { x: 1100.0, y: 20.0 })
                                .resizable(false)
                                .show(ctx, |ui| {
                                    generator_control(&self.store, ui, generator_index);
                                    ui.separator();
                                    ui.label("Envelope");
                                    envelope_control(&self.store, ui, generator_index);
                                });
                        }
                    }
                    if self.window_state.effects {
                        let mixer = &self.store.get().project.mixer;
                        for (mixer_index, mixer) in mixer.iter().enumerate() {
                            for effect_index in 0..mixer.effects.len() {
                                effect_control(
                                    ctx,
                                    &mut self.window_state,
                                    &self.store,
                                    mixer_index,
                                    effect_index,
                                );
                            }
                        }
                    }
                    if self.window_state.scale {
                        egui::Window::new("Scale")
                            .open(&mut self.window_state.scale)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                key_control(&self.store, ui);
                            });
                    }
                    if self.window_state.manual_notes {
                        egui::Window::new("Notes")
                            .open(&mut self.window_state.manual_notes)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .resizable(false)
                            .show(ctx, |ui| {
                                track_control(&self.store, ui);
                            });
                    }

                    if self.window_state.note_roll {
                        egui::Window::new("Note roll")
                            .open(&mut self.window_state.note_roll)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .show(ctx, |ui| {
                                note_roll_display(&self.store, ui);
                            });
                    }

                    egui::Window::new("Toolbar")
                        .default_pos(Pos2 { x: 600.0, y: 20.0 })
                        .resizable(false)
                        .show(ctx, |ui| {
                            let volume = self.store.get().volume;
                            knob(
                                ui,
                                "Volume",
                                volume,
                                |it| self.store.dispatchr(Action::SetVolume(it)),
                                FloatRange(0.0, 1.0),
                            );

                            let bpm = self.store.get().project.bpm as f64;
                            ui.add(
                                egui::Slider::from_get_set(
                                    20.0..=200.0,
                                    get_set(bpm, |it| {
                                        self.store.dispatchr(Action::SetBpm(it as Beats))
                                    }),
                                )
                                .text("BPM")
                                .logarithmic(true),
                            );
                            undo_redo_control(&mut self.store, ui);
                            ui.separator();
                            play_control(
                                &self.store,
                                &mut self.async_state,
                                &mut self.audio_state,
                                ui,
                            );
                            ui.separator();
                            sample_control(
                                &self.store,
                                &mut self.audio_state,
                                &mut self.async_state,
                                ui,
                            );
                        });

                    ui.separator();
                    track_placement_control(&self.store, ui);

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        egui::warn_if_debug_build(ui);
                    });
                });
        });
    }
}
