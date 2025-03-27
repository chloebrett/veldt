use super::app::App;
use super::audio_vis::audio_vis;
use crate::audio_player::play;
use crate::rpc::render as server_render;
use egui::Ui;
use mesic::SAMPLE_RATE;
use mesic::render as local_render;
use mesic::{AmpNode, Sig};
use poll_promise::Promise;

pub fn play_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play (local)").clicked() {
        let volume = app.store.get().volume;
        let sample_count = SAMPLE_RATE as u32 * 10; // 10 seconds
        let audio_node = local_render(&app.store.get().project);
        let mut clipped_audio = AmpNode {
            input: audio_node,
            volume,
            should_clip: true,
        };
        app.audio = clipped_audio.buffer(sample_count);

        let signal = dasp_signal::from_iter(app.audio.clone());
        app.handle = Some(play(signal));
    }
    if let Some(render_promise) = &app.server_render_promise {
        if let Some(Some(server_audio)) = render_promise.ready() {
            if ui.button("Play (server)").clicked() {
                let _volume = app.store.get().volume; // TODO: reconnect
                app.audio = server_audio.to_vec();
                let signal = dasp_signal::from_iter(app.audio.clone());
                app.handle = Some(play(signal));
            }
        }
    }
    if ui.button("Load audio (server)").clicked() {
        let project = app.store.get().project.clone();
        // TODO: use an action.
        app.server_render_promise =
            Some(Promise::spawn_local(
                async move { server_render(project).await },
            ))
    }
    audio_vis(app, ui);
}
