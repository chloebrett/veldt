use super::{FrequencyDisplay, audio_vis::audio_vis};
use crate::audio_player::AudioPlayer;
use crate::promise::{poll, spawn};
use crate::rpc::render as server_render;
use crate::view::View;
use crate::{AsyncState, AudioState};
use dasp_frame::Stereo;
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
        graph.add_output_node(AmpNode {
            volume,
            should_clip: true,
        });
        let player = AudioPlayer::new(graph, audio_state.pre_render);
        audio_state.player = Some(player);
        audio_state.player.as_mut().unwrap().play();
    }
    poll(
        &mut async_state.server_render,
        |audio: &Vec<Stereo<f32>>| {
            let volume = store.get().volume;
            audio_state.audio = audio.to_vec();
            let mut graph = RenderGraph::from_vec(audio_state.audio.clone());
            graph.add_output_node(AmpNode {
                volume,
                should_clip: true,
            });
            let player = AudioPlayer::new(graph, audio_state.pre_render);
            audio_state.player = Some(player);
            audio_state.player.as_mut().unwrap().play();
        },
    );
    if ui.button("Load audio (server)").clicked() {
        let project = store.get().project.clone();
        spawn(&mut async_state.server_render, async move {
            server_render(project).await
        })
    }
    ui.checkbox(&mut audio_state.pre_render, "Pre-render audio");
    audio_vis(audio_state, ui);
    FrequencyDisplay::new(audio_state).ui(ui)
}
