use super::app::{AsyncState, AudioState};
use super::audio_vis::audio_vis;
use crate::audio_player::play;
use crate::rpc::render as server_render;
use egui::Ui;
use mesic::graph::{AmpNode, RenderGraph};
use mesic::render as local_render;
use poll_promise::Promise;
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
        audio_state.handle = Some(play(graph));
    }
    if let Some(render_promise) = &async_state.server_render {
        if let Some(Some(server_audio)) = render_promise.ready() {
            if ui.button("Play (server)").clicked() {
                let volume = store.get().volume;
                audio_state.audio = server_audio.to_vec();
                let mut graph = RenderGraph::from_vec(audio_state.audio.clone());
                graph.add_node(AmpNode {
                    volume,
                    should_clip: true,
                });
                audio_state.handle = Some(play(graph));
            }
        }
    }
    if ui.button("Load audio (server)").clicked() {
        let project = store.get().project.clone();
        // TODO: use an action.
        async_state.server_render =
            Some(Promise::spawn_local(
                async move { server_render(project).await },
            ))
    }
    audio_vis(audio_state, ui);
}
