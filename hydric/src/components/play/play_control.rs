use super::{FrequencyDisplay, audio_vis::audio_vis};
use crate::promise::{poll, spawn};
use crate::rpc::render as server_render;
use crate::view::View;
use crate::widget::checkbox;
use crate::{AsyncState, AudioState};
use dasp_frame::Stereo;
use egui::Ui;
use state::Store;

pub fn play_control(
    store: &Store,
    async_state: &mut AsyncState,
    audio_state: &mut AudioState,
    ui: &mut Ui,
) {
    let player = &mut audio_state.player;
    if ui.button("Set audio from local + play").clicked() {
        let volume = store.get().volume;
        player.set_project(Box::new(store.get().project.clone()), volume);
        player.play();
    }

    ui.separator();

    if ui.button("Set audio from local").clicked() {
        let volume = store.get().volume;
        player.set_project(Box::new(store.get().project.clone()), volume);
    }

    if ui.button("Set audio from server").clicked() {
        let project = store.get().project.clone();
        spawn(&mut async_state.server_render, async move {
            server_render(project).await
        })
    }
    poll(
        &mut async_state.server_render,
        |audio: &Vec<Stereo<f32>>| {
            let volume = store.get().volume;
            audio_state.audio = audio.to_vec();
            player.set_audio(audio_state.audio.clone(), volume);
        },
    );

    if ui.button("Play").clicked() {
        player.play();
    }
    if ui.button("Pause").clicked() {
        player.pause();
    }
    if ui.button("Stop").clicked() {
        player.pause();
        player.seek(0);
    }
    checkbox(ui, player.is_looping(), |it| player.set_looping(it), "Loop");

    audio_vis(audio_state, ui);
    FrequencyDisplay::new(audio_state).ui(ui)
}
