use super::{FrequencyDisplay, audio_vis::audio_vis};
use crate::promise::{poll, spawn};
use crate::rpc::render as server_render;
use crate::view::View;
use crate::{AsyncState, AudioState};
use dasp_frame::Stereo;
use egui::Ui;
use mesic::SAMPLE_RATE;
use state::Store;

pub fn play_control(
    store: &Store,
    async_state: &mut AsyncState,
    audio_state: &mut AudioState,
    ui: &mut Ui,
) {
    if ui.button("Play (local)").clicked() {
        let volume = store.get().volume;
        audio_state.player.init();
        audio_state
            .player
            .set_project(Box::new(store.get().project.clone()), volume);
        audio_state.player.play();
    }
    if ui.button("Pause").clicked() {
        audio_state.player.pause();
    }
    if ui.button("Resume").clicked() {
        audio_state.player.play();
    }
    if ui.button("Seek to 0").clicked() {
        audio_state.player.seek(0);
    }
    if ui.button("Seek to 1 sec").clicked() {
        audio_state.player.seek(SAMPLE_RATE as usize);
    }
    poll(
        &mut async_state.server_render,
        |audio: &Vec<Stereo<f32>>| {
            let volume = store.get().volume;
            audio_state.audio = audio.to_vec();
            audio_state.player.init();
            audio_state
                .player
                .set_audio(audio_state.audio.clone(), volume);
            audio_state.player.play();
        },
    );
    if ui.button("Load audio (server)").clicked() {
        let project = store.get().project.clone();
        spawn(&mut async_state.server_render, async move {
            server_render(project).await
        })
    }
    audio_vis(audio_state, ui);
    FrequencyDisplay::new(audio_state).ui(ui)
}
