use super::app::App;
use super::audio_vis::audio_vis;
use crate::audio_player::play;
use crate::rpc::render as server_render;
use dasp_signal::Signal;
use egui::Ui;
use mesic::render_signal as local_render;
use poll_promise::Promise;

pub fn play_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play (local)").clicked() {
        let volume = app.store.get().volume;
        let render_output = local_render(&app.store.get().project);
        app.audio = render_output.clone().until_exhausted().collect();
        app.handle = Some(play(Box::new(
            render_output.clone().scale_amp(volume).clip_amp(1.0),
        )));
    }
    if let Some(render_promise) = &app.server_render_promise {
        if let Some(Some(server_audio)) = render_promise.ready() {
            if ui.button("Play (server)").clicked() {
                let volume = app.store.get().volume;
                app.audio = server_audio.to_vec();
                let signal = dasp_signal::from_iter(app.audio.clone().into_iter()).scale_amp(volume).clip_amp(1.0);
                app.handle = Some(play(Box::new(signal)));
            }
        }
    }
    if ui.button("Load audio (server)").clicked() {
        let project = app.store.get().project.clone();
        app.server_render_promise =
            Some(Promise::spawn_local(
                async move { server_render(project).await },
            ))
    }
    audio_vis(app, ui);
}
