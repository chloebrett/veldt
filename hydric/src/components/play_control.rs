use super::app::App;
use super::audio_vis::audio_vis;
use crate::audio_player::play;
use crate::rpc::render as server_render;
use egui::Ui;
use mesic::render as local_render;
use poll_promise::Promise;

pub fn play_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play (local)").clicked() {
        app.audio = local_render(&app.store.project)
            .into_iter()
            .map(|sample| sample.clamp(-1.0, 1.0))
            .collect();
        app.handle = Some(play(&app.audio));
    }
    if let Some(render_promise) = &app.server_render_promise {
        if let Some(Some(server_audio)) = render_promise.ready() {
            if ui.button("Play (server)").clicked() {
                app.audio = server_audio
                    .to_vec()
                    .iter()
                    .copied()
                    .map(|sample| sample.clamp(-1.0, 1.0))
                    .collect::<Vec<f32>>();
                app.handle = Some(play(&app.audio))
            }
        }
    }
    if ui.button("Load audio (server)").clicked() {
        let track = app.store.project.tracks[0].clone();
        let generator = app.store.project.generators[0].clone();
        let mixer_channel = app.store.project.mixer[0].effects.clone();
        let bpm = app.store.project.bpm;
        app.server_render_promise = Some(Promise::spawn_local(async move {
            server_render(track, mixer_channel, generator, bpm).await
        }))
    }
    audio_vis(app, ui);
}
