use super::{
    KeyView, NoteRoll, NoteView, PlacementView, TrackRoll,
    effect::{EffectView, MixerView},
    generator::{GeneratorView, generators_control},
    menu::MenuBar,
    play::{SampleTreeView, ToolbarView},
};
use crate::promise::spawn;
use crate::rpc::broadcast_actions;
use crate::rpc::load_project_list;
use crate::view::View;
use crate::{AsyncState, LocalState, WindowState, playback::AudioPlayer};
use crate::{components::FrameHistory, window_state::WindowState2};
use egui::{ScrollArea, Ui, scroll_area::ScrollBarVisibility};
use mesic::graph::RenderGraph;
use poll_promise::Promise;
use state::{Action, EffectSelector, GeneratorSelector, Store};
use std::sync::mpsc::channel;

pub struct App {
    // State used for rendering audio and/or by other users.
    // Example: knob positions.
    pub store: Store,
    // State only used for rendering local UIs.
    // Example: selected notes in the note roll.
    pub local_state: LocalState,
    pub frame_history: FrameHistory,
    pub async_state: AsyncState,
    pub player: AudioPlayer,
    pub window_state: WindowState,
    pub window_state2: WindowState2,
}

impl Default for App {
    fn default() -> Self {
        let broadcast = |actions| {
            let _ = Promise::spawn_local(broadcast_actions(actions));
        };
        let (tx, rx) = channel();
        let store = Store::new(broadcast, tx);
        let graph = RenderGraph::new(store.get(), rx);

        App {
            store,
            local_state: LocalState::default(),
            frame_history: FrameHistory::default(),
            async_state: AsyncState::default(),
            player: AudioPlayer::new(graph),
            window_state: WindowState::default(),
            window_state2: WindowState2::default(),
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

        self.player.maybe_update();

        egui::TopBottomPanel::top("veldt_menu").show(ctx, |ui| {
            MenuBar::new(
                &mut self.store,
                &mut self.player,
                &mut self.window_state,
                &mut self.window_state2,
                &mut self.async_state,
            )
            .ui(ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    self.ui(ui);

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

        for sel in self.visible_generators() {
            let generators = &mut self.window_state.generators;
            let visible = generators.get(sel);
            GeneratorView::new(
                &self.store,
                &sel,
                &self.local_state,
                &mut self.player,
                visible,
                || generators.set(sel, false),
            )
            .ui(ui);
        }
        if self.window_state.mixer.visible {
            MixerView::new(
                &mut self.window_state,
                &self.store,
                &self.local_state,
                &self.player,
            )
            .ui(ui);
        }

        ToolbarView::new(&mut self.store, &mut self.async_state, &mut self.player).ui(ui);

        for effect_selector in self.visible_effects() {
            let dispatch = |action| self.store.dispatch(&effect_selector, action);
            let on_release = || self.store.dispatchr(Action::Release);
            if let Some(mut it) = EffectView::new(
                &self.store,
                &effect_selector,
                &mut self.window_state,
                dispatch,
                on_release,
            ) {
                it.ui(ui)
            }
        }
        if self.window_state.scale {
            let dispatch = |action| self.store.dispatchr(action);
            let key = self.store.get().key;
            let scale = self.store.get().scale;
            KeyView::new(dispatch, &mut self.window_state.scale, key, scale).ui(ui);
        }

        NoteView::new(&self.store, &self.local_state).ui(ui);
        NoteRoll::new(&self.store, &self.local_state, &mut self.player).ui(ui);
        PlacementView::new(&self.store, &self.local_state).ui(ui);

        SampleTreeView::new(
            &self.store,
            &mut self.async_state,
            &mut self.window_state.sample_tree,
            &mut self.player
        )
        .ui(ui);
        TrackRoll::new(&self.store, &mut self.window_state2, &self.local_state).ui(ui);
    }
}
