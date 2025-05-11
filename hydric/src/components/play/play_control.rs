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
    ui.horizontal(|ui| {
        if ui.button("▶").clicked() {
            player.play();
        }
        if ui.button("⏸").clicked() {
            player.pause();
        }
        if ui.button("◼").clicked() {
            player.pause();
            player.seek(0);
        }
        checkbox(ui, player.is_looping(), |it| player.set_looping(it), "Loop");

        ui.label(player.current_time());
    });

    ui.separator();

    ui.horizontal(|ui| {
        // TODO: create a debug options dropdown in the main menu and put this there.
        if ui.button("Recreate mixer (for debug)").clicked() {
            player.refresh_mixer();
        }
        if ui.button("Set audio from server").clicked() {
            let project = store.get().project.clone();
            spawn(&mut async_state.server_render, async move {
                server_render(project).await
            })
        }
    });

    ui.separator();

    poll(
        &mut async_state.server_render,
        |audio: &Vec<Stereo<f32>>| {
            audio_state.audio = audio.to_vec();
            player.set_audio(audio_state.audio.clone());
        },
    );

    audio_vis(audio_state, ui);
    FrequencyDisplay::new(Some(audio_state), None).ui(ui)
}
