use super::{
    ChannelEffectView, KeyView, NoteRoll, NoteView, PlacementView, TrackRoll,
    effect::{EffectView, MixerView},
    generator::{GeneratorView, generators_control},
    menu::MenuBar,
    play::{MicrophoneView, SampleTreeView, ToolbarView},
};
use crate::components::FrameHistory;
use crate::playback::Microphone;
use crate::rpc::broadcast_actions;
use crate::rpc::load_project_list;
use crate::view::View;
use crate::{AsyncState, LocalState, playback::AudioPlayer};
use crate::{promise::spawn, window_state::WindowKind};
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
    pub mic: Microphone,
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
            mic: Microphone::new(),
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
        self.local_state.window_state.visible_generators()
    }

    fn visible_effects(&self) -> Vec<EffectSelector> {
        self.local_state.window_state.visible_effects()
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
        self.mic.update_mic_recording();

        self.local_state.window_state.update(&self.store);

        egui::TopBottomPanel::top("veldt_menu").show(ctx, |ui| {
            MenuBar::new(
                &mut self.store,
                &self.local_state,
                &mut self.player,
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
        if self
            .local_state
            .window_state
            .get_visible(WindowKind::GeneratorList)
        {
            generators_control(ui, &self.local_state, &self.store);
        }

        for sel in self.visible_generators() {
            GeneratorView::new(&self.store, &sel, &self.local_state, &mut self.player).ui(ui);
        }
        if self.local_state.window_state.get_visible(WindowKind::Mixer) {
            MixerView::new(&self.store, &self.local_state, &self.player).ui(ui);
        }

        ToolbarView::new(&mut self.store, &mut self.async_state, &mut self.player).ui(ui);

        for effect_selector in self.visible_effects() {
            let dispatch = |action| self.store.dispatch(&effect_selector, action);
            let on_release = || self.store.dispatchr(Action::Release);
            if let Some(mut it) = EffectView::new(
                &self.store,
                &effect_selector,
                &self.local_state,
                dispatch,
                on_release,
            ) {
                it.ui(ui)
            }
        }
        if self.local_state.window_state.get_visible(WindowKind::Scale) {
            let dispatch = |action| self.store.dispatchr(action);
            let key = self.store.get().key;
            let scale = self.store.get().scale;
            KeyView::new(dispatch, &self.local_state, key, scale).ui(ui);
        }

        ChannelEffectView::new(&self.store, &self.local_state).ui(ui);

        NoteView::new(&self.store, &self.local_state).ui(ui);
        NoteRoll::new(&self.store, &self.local_state, &mut self.player).ui(ui);

        MicrophoneView::new(&self.local_state, &mut self.mic).ui(ui);
        PlacementView::new(&self.store, &self.local_state).ui(ui);

        SampleTreeView::new(
            &self.store,
            &mut self.async_state,
            &mut self.player,
            &self.local_state,
        )
        .ui(ui);
        TrackRoll::new(&self.store, &self.local_state).ui(ui);
    }
}
