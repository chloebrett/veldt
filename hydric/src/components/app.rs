use super::{
    effect::{effect_control, mixer_control},
    generator::{generator_control, generators_control},
    key_control::KeyControl,
    load_control,
    note_roll::NoteRoll,
    play::{play_control, sample_control},
    save_button, toggle_window_panel, track_control, track_placement_control,
    track_roll::TrackRoll,
    undo_redo_control,
};
use crate::widget::{default_window, knob, slider, string_observer};
use crate::{audio_player::Handle, promise::AsyncResult, view::View};
use egui::Pos2;
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use shared::model::{GeneratorType, Project, Sample};
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

pub struct MixerWindowState {
    pub visible: bool,
    // Currently active / shown channel.
    pub channel: usize,
}

/// Which windows are currently shown.
pub struct WindowState {
    pub mixer: MixerWindowState,
    pub effects: Vec<Vec<bool>>, // by ID (within each mixer)
    pub manual_notes: bool,
    pub generator_list: bool,
    pub generators: Vec<bool>, // by ID
    pub scale: bool,
    pub note_roll: bool,
}

impl Default for WindowState {
    fn default() -> WindowState {
        // TODO: generate this automatically from the project state.
        WindowState {
            mixer: MixerWindowState {
                visible: false,
                channel: 0,
            },
            effects: vec![vec![false, false, false, false]],
            manual_notes: false,
            generator_list: false,
            generators: vec![false],
            scale: false,
            note_roll: false,
        }
    }
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
        // This applies all of the pending actions. It avoids cloning the state.
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

                    if self.window_state.generator_list {
                        generators_control(ctx, &mut self.window_state, &self.store);
                    }

                    let generators = &self.store.get().project.generators;
                    for (generator_index, generator) in generators.iter().enumerate() {
                        if self.window_state.generators[generator_index] {
                            // TODO: move this to generator_control.rs.
                            let title = match &generator.kind {
                                GeneratorType::SimpleWave { .. } => "Simple Wave Generator",
                                GeneratorType::Noise { .. } => "Noise Generator",
                            };
                            default_window(title)
                                .open(&mut self.window_state.generators[0])
                                .default_pos(Pos2 { x: 1100.0, y: 20.0 })
                                .show(ctx, |ui| {
                                    generator_control(&self.store, ui, generator_index);
                                });
                        }
                    }
                    if self.window_state.mixer.visible {
                        mixer_control(ctx, &mut self.window_state, &self.store);
                    }
                    let mixer = &self.store.get().project.mixer;
                    for (mixer_index, channel) in mixer.iter().enumerate() {
                        for effect_index in 0..channel.effects.len() {
                            if *self.window_state.effects[mixer_index]
                                .get(effect_index)
                                .unwrap_or(&false)
                            {
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
                        default_window("Scale")
                            .open(&mut self.window_state.scale)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .show(ctx, |ui| {
                                KeyControl::new(self.store.get().key, self.store.get().scale)
                                    .ui(&self.store, ui);
                            });
                    }
                    if self.window_state.manual_notes {
                        default_window("Notes")
                            .open(&mut self.window_state.manual_notes)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .show(ctx, |ui| {
                                track_control(&self.store, ui);
                            });
                    }

                    if self.window_state.note_roll {
                        default_window("Note roll")
                            .open(&mut self.window_state.note_roll)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .resizable(true)
                            .show(ctx, |ui| {
                                let track_index = 0;
                                NoteRoll::new(track_index).ui(&self.store, ui);
                            });
                    }

                    default_window("Toolbar")
                        .default_pos(Pos2 { x: 600.0, y: 20.0 })
                        .show(ctx, |ui| {
                            let on_release = || self.store.dispatchr(Action::Release);

                            let volume = self.store.get().volume;
                            knob(
                                ui,
                                "Volume",
                                volume,
                                |it| self.store.dispatchr(Action::SetVolume(it)),
                                0.0..=1.0,
                                on_release,
                            );

                            let bpm = self.store.get().project.bpm as f64;
                            slider(
                                ui,
                                "BPM",
                                bpm,
                                |it| self.store.dispatchr(Action::SetBpm(it as Beats)),
                                20.0..=200.0,
                                on_release,
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
                    ui.separator();
                    TrackRoll::new().ui(&self.store, ui);

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        egui::warn_if_debug_build(ui);
                    });
                });
        });
    }
}
