use super::app::App;
use super::audio_vis::audio_vis;
use crate::audio_player::play;
use crate::rpc::render;
use egui::Ui;
use mesic::{create_track, render as local_render};
use poll_promise::Promise;

pub fn play_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play (local)").clicked() {
        let track = create_track(app.notes.clone());
        app.audio = local_render(&track, app.effects.clone(), app.generator.clone(), app.bpm)
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
        let track = create_track(app.notes.clone());
        let generator = app.generator.clone();
        let effects = app.effects.clone();
        let bpm = app.bpm;
        app.server_render_promise = Some(Promise::spawn_local(async move {
            render(track, effects, generator, bpm).await
        }))
    }
    audio_vis(app, ui);
}
