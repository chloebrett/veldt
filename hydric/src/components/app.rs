use super::{
    KeyView, SaveLoadView,
    effect::{EffectWindow, MixerWindow},
    generator::{generator_control, generators_control},
    menu::Menu,
    note_control::NoteControl,
    note_roll::NoteRoll,
    play::{SampleTreeWindow, play_control, sample_control},
    track_placement_control,
    track_roll::TrackRoll,
    undo_redo_control,
};
use crate::components::FrameHistory;
use crate::promise::spawn;
use crate::rpc::broadcast_actions;
use crate::rpc::{load_project_list, load_sample_tree};
use crate::view::View;
use crate::view::WindowView;
use crate::widget::{default_window, get_set, knob, slider, string_observer};
use crate::{AsyncState, AudioState, WindowState};
use egui::{Id, Pos2, pos2};
use egui::{ScrollArea, scroll_area::ScrollBarVisibility};
use poll_promise::Promise;
use shared::model::GeneratorType;
use shared::types::Beats;
use state::{Action, FloatField, Selector, Store, TypeField};

pub struct App {
    pub store: Store,
    pub frame_history: FrameHistory,
    pub async_state: AsyncState,
    pub audio_state: AudioState,
    pub window_state: WindowState,
}

impl Default for App {
    fn default() -> Self {
        let broadcast = |actions| {
            let _ = Promise::spawn_local(broadcast_actions(actions));
        };
        App {
            store: Store::new(broadcast),
            frame_history: FrameHistory::default(),
            async_state: AsyncState::default(),
            audio_state: AudioState::default(),
            window_state: WindowState::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut app = App::default();

        spawn(&mut app.async_state.project_list, async move {
            load_project_list().await
        });

        spawn(&mut app.async_state.load_sample_tree, async move {
            load_sample_tree().await
        });

        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Snapshot the state at the start of each frame.
        // This applies all of the pending actions. It avoids cloning the state.
        self.store.snapshot();

        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        egui::CentralPanel::default().show(ctx, |ui| {
            Menu::new(&mut self.store, &mut self.window_state).ui(ui);
            ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let project_name = self.store.get().project.name.clone();
                        let mut name_observer = string_observer(
                            get_set(project_name.clone(), |it| {
                                self.store
                                    .dispatchr(Action::SetChild(TypeField::ProjectName(it)))
                            }),
                            project_name.clone(),
                        );
                        ui.text_edit_singleline(&mut name_observer);
                        SaveLoadView::new(&self.store, &mut self.async_state).ui(ui);
                    });

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
                                GeneratorType::SubSynth { .. } => "Subtractive Synth",
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
                        MixerWindow::new(&mut self.window_state, &self.store).ui(ctx);
                    }
                    let mixer = &self.store.get().project.mixer;
                    for (mixer_index, channel) in mixer.iter().enumerate() {
                        for effect_index in 0..channel.effects.len() {
                            if *self.window_state.effects[mixer_index]
                                .get(effect_index)
                                .unwrap_or(&false)
                            {
                                let sel = Selector::Effect(mixer_index, effect_index);
                                let dispatch = |action| self.store.dispatch(&sel, action);
                                let on_release = || self.store.dispatchr(Action::Release);
                                EffectWindow::new(
                                    &self.store,
                                    mixer_index,
                                    effect_index,
                                    &mut self.window_state,
                                    dispatch,
                                    on_release,
                                )
                                .ui(ctx);
                            }
                        }
                    }
                    if self.window_state.scale {
                        default_window("Scale")
                            .open(&mut self.window_state.scale)
                            .default_pos(Pos2 { x: 600.0, y: 20.0 })
                            .show(ctx, |ui| {
                                let dispatch = |action| self.store.dispatchr(action);
                                KeyView::new(
                                    &dispatch,
                                    self.store.get().key,
                                    self.store.get().scale,
                                )
                                .ui(ui);
                            });
                    }

                    let note_id = Id::new("note_window");
                    if ui.data_mut(|data| *data.get_temp_mut_or(note_id, false)) {
                        let mut open = true;
                        let active_track = ui.data_mut(|data| {
                            let id = Id::new("active_track_index");
                            *data.get_temp_mut_or(id, None)
                        });
                        let active_note = ui.data_mut(|data| {
                            let id = Id::new("active_note_index");
                            *data.get_temp_mut_or(id, None)
                        });
                        if let Some(track_index) = active_track {
                            if let Some(note_index) = active_note {
                                default_window("Notes")
                                    .open(&mut open)
                                    .default_pos(Pos2 { x: 600.0, y: 20.0 })
                                    .show(ctx, |ui| {
                                        NoteControl::new(&self.store, track_index, note_index)
                                            .ui(ui);
                                    });
                            }
                        }
                        ui.data_mut(|data| {
                            // Check if state has been changed within component as well as with
                            // x'ing out of window.
                            data.insert_temp(
                                note_id,
                                data.get_temp(note_id).unwrap_or(false) && open,
                            );
                        })
                    };

                    let note_roll_id = Id::new("note_roll_window");
                    if ui.data_mut(|data| {
                        *data.get_temp_mut_or_insert_with(note_roll_id, move || false)
                    }) {
                        let active_track = ui.data_mut(|data| {
                            let id = Id::new("active_track_index");
                            *data.get_temp_mut_or(id, None)
                        });
                        if let Some(track_index) = active_track {
                            let mut open = true;
                            default_window(&format!("Track: {}", track_index))
                                .open(&mut open)
                                .default_pos(Pos2 { x: 600.0, y: 20.0 })
                                .resizable(true)
                                .show(ctx, |ui| {
                                    NoteRoll::new(&self.store, track_index).ui(ui);
                                });
                            ui.data_mut(|data| {
                                data.insert_temp(note_roll_id, open);
                            })
                        }
                    }

                    SampleTreeWindow::new(
                        &self.store,
                        &mut self.async_state,
                        &mut self.window_state.sample_tree,
                    )
                    .ui(ui);

                    default_window("Toolbar")
                        .default_pos(Pos2 { x: 600.0, y: 20.0 })
                        .show(ctx, |ui| {
                            let on_release = || self.store.dispatchr(Action::Release);

                            let volume = self.store.get().volume;
                            knob(
                                ui,
                                "Volume",
                                volume,
                                |it| {
                                    self.store
                                        .dispatchr(Action::SetFloat(FloatField::Volume, it))
                                },
                                0.0..=1.0,
                                /* neutral= */ 1.0,
                                on_release,
                            );

                            let bpm = self.store.get().project.bpm as f64;
                            slider(
                                ui,
                                "BPM",
                                bpm,
                                |it| {
                                    self.store
                                        .dispatchr(Action::SetFloat(FloatField::Bpm, it as Beats))
                                },
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
                    default_window("Track Roll")
                        .default_pos(pos2(30.0, 200.0))
                        .resizable(true)
                        .open(&mut self.window_state.track_roll)
                        .show(ctx, |ui| {
                            TrackRoll::new(&self.store).ui(ui);
                        });

                    let placement_window_id = Id::new("track_placement_window");
                    if ui.data_mut(|data| {
                        *data.get_temp_mut_or_insert_with(placement_window_id, move || false)
                    }) {
                        let active_track_placement = ui.data_mut(|data| {
                            let id = Id::new("active_track_placement_index");
                            *data.get_temp_mut_or::<Option<usize>>(id, None)
                        });
                        if let Some(placement_index) = active_track_placement {
                            let mut open = true;
                            default_window(&format!("Track Placement {}", placement_index))
                                .open(&mut open)
                                .default_pos(Pos2 { x: 100.0, y: 20.0 })
                                .resizable(true)
                                .show(ctx, |ui| {
                                    track_placement_control(&self.store, ui);
                                });
                            ui.data_mut(|data| {
                                data.insert_temp(placement_window_id, open);
                            })
                        }
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        self.frame_history.ui(ui);
                    });
                });
        });
    }
}
