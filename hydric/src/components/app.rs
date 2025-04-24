use super::{
    KeyView, NoteRoll, NoteView, SaveLoadView, TrackPlacementView, TrackRoll,
    menu::MenuBar,
    effect::{EffectView, MixerView},
    generator::{generator_control, generators_control},
    play::{SampleTreeView, ToolbarView},
};
use crate::components::FrameHistory;
use crate::promise::spawn;
use crate::rpc::broadcast_actions;
use crate::rpc::load_project_list;
use crate::view::View;
use crate::{AsyncState, AudioState, WindowState};
use crate::{EffectSelector, GeneratorSelector};
use egui::{ScrollArea, Ui, scroll_area::ScrollBarVisibility};
use mesic::graph::RenderGraph;
use poll_promise::Promise;
use state::{Action, Selector, Store};
use std::sync::mpsc::channel;

pub struct App {
    pub store: Store,
    graph: RenderGraph,
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
        let mut graph = RenderGraph::default();
        let (tx, rx) = channel();
        graph.set_receiver(rx);
        App {
            store: Store::new(broadcast, tx),
            graph,
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

    fn visible_generators(&self) -> Vec<GeneratorSelector> {
        self.window_state.generators.clone().as_vec()
    }

    fn visible_effects(&self) -> Vec<EffectSelector> {
        self.window_state.effects.clone().as_vec()
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
            MenuBar::new(&mut self.store, &mut self.window_state, &mut self.async_state).ui(ui);
            ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    self.ui(ui);

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

impl View for App {
    fn ui(&mut self, ui: &mut Ui) {
        if self.window_state.generator_list {
            generators_control(ui.ctx(), &mut self.window_state, &self.store);
        }

        for generator_index in self.visible_generators() {
            // TODO: make a GeneratorView.
            let visible = self.window_state.generators.get(generator_index);
            generator_control(&self.store, ui, generator_index, visible, || {
                self.window_state.generators.set(generator_index, false)
            });
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

        for (mixer_index, effect_index) in self.visible_effects() {
            let dispatch = |action| {
                self.store
                    .dispatch(&Selector::Effect(mixer_index, effect_index), action)
            };
            let on_release = || self.store.dispatchr(Action::Release);
            EffectView::new(
                &self.store,
                mixer_index,
                effect_index,
                &mut self.window_state,
                dispatch,
                on_release,
            )
            .map(|mut it| it.ui(ui));
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
