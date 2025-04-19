use super::audio_vis::audio_vis;
use crate::audio_player::play;
use crate::promise::{poll, spawn};
use crate::rpc::render as server_render;
use crate::{AsyncState, AudioState};
use egui::Ui;
use mesic::graph::{AmpNode, RenderGraph};
use mesic::render as local_render;
use state::Store;

pub fn play_control(
    store: &Store,
    async_state: &mut AsyncState,
    audio_state: &mut AudioState,
    ui: &mut Ui,
) {
    if ui.button("Play (local)").clicked() {
        let volume = store.get().volume;
        let mut graph = local_render(&store.get().project);
        graph.add_node(AmpNode {
            volume,
            should_clip: true,
        });
        audio_state.handle = Some(play(graph, audio_state.pre_render));
    }
    poll(&mut async_state.server_render, |audio: &Vec<f32>| {
        let volume = store.get().volume;
        // TODO: visualise both channels, not just the left.
        audio_state.audio = audio.to_vec();
        let mut graph = RenderGraph::from_vec(audio_state.audio.clone());
        graph.add_node(AmpNode {
            volume,
            should_clip: true,
        });
        audio_state.handle = Some(play(graph, audio_state.pre_render));
    });
    if ui.button("Load audio (server)").clicked() {
        let project = store.get().project.clone();
        spawn(&mut async_state.server_render, async move {
            server_render(project).await
        })
    }
    ui.checkbox(&mut audio_state.pre_render, "Pre-render audio");
    audio_vis(audio_state, ui);
}
