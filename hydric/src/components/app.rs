use super::{
    KeyView, NoteRoll, NoteView, SaveLoadView, TrackPlacementView, TrackRoll,
    effect::{EffectView, MixerView},
    generator::{generator_control, generators_control},
    menu::Menu,
    play::{SampleTreeView, ToolbarView},
};
use crate::components::FrameHistory;
use crate::promise::spawn;
use crate::rpc::broadcast_actions;
use crate::rpc::load_project_list;
use crate::view::View;
use crate::{AsyncState, AudioState, WindowState};
use egui::{ScrollArea, Ui, scroll_area::ScrollBarVisibility};
use poll_promise::Promise;
use shared::serialize::map_vec;
use state::{Action, Selector, Store};

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

        app
    }

    fn visible_generators(&self) -> Vec<usize> {
        let generators = &self.store.get().project.generators;
        map_vec(
            (0..generators.len())
                .filter(|it| self.window_state.generators[*it])
                .collect(),
        )
    }

    fn visible_effects(&self) -> Vec<Selector> {
        let mixer = &self.store.get().project.mixer;
        mixer
            .iter()
            .enumerate()
            .flat_map(|(mi, channel)| {
                (0..channel.effects.len())
                    .filter(move |ei| {
                        *self.window_state.effects[mi]
                            .get(*ei)
                            .unwrap_or(&false)
                    })
                    .map(move |ei| Selector::Effect(mi, ei))
            })
            .collect()
    }

    fn windows(&mut self, ui: &mut Ui) {
        if self.window_state.generator_list {
            generators_control(ui.ctx(), &mut self.window_state, &self.store);
        }

        for generator_index in self.visible_generators() {
            // TODO: make a GeneratorView.
            generator_control(
                &self.store,
                ui,
                generator_index,
                &mut self.window_state.generators[generator_index],
            );
        }
        if self.window_state.mixer.visible {
            MixerView::new(&mut self.window_state, &self.store).ui(ui);
        }

        ToolbarView::new(
            &mut self.store,
            &mut self.async_state,
            &mut self.audio_state,
        )
        .ui(ui);

        for sel in self.visible_effects() {
            let dispatch = |action| self.store.dispatch(&sel, action);
            let on_release = || self.store.dispatchr(Action::Release);
            EffectView::new(
                &self.store,
                &sel,
                &mut self.window_state,
                dispatch,
                on_release,
            )
            .ui(ui);
        }
        if self.window_state.scale {
            let dispatch = |action| self.store.dispatchr(action);
            let key = self.store.get().key;
            let scale = self.store.get().scale;
            KeyView::new(dispatch, &mut self.window_state.scale, key, scale).ui(ui);
        }

        NoteView::new(&self.store).ui(ui);
        NoteRoll::new(&self.store).ui(ui);
        TrackPlacementView::new(&self.store).ui(ui);

        SampleTreeView::new(
            &self.store,
            &mut self.async_state,
            &mut self.window_state.sample_tree,
        )
        .ui(ui);
        TrackRoll::new(&self.store, &mut self.window_state).ui(ui);
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
                    self.windows(ui);

                    // TODO: put this behind a window.
                    ui.horizontal(|ui| {
                        SaveLoadView::new(&self.store, &mut self.async_state).ui(ui);
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        self.frame_history.ui(ui);
                    });
                });
        });
    }
}
