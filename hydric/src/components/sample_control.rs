use super::app::App;
use crate::audio_player::play;
use egui::Ui;
use crate::state::Action;

pub fn sample_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play sample").clicked() {
        let sample = app.store.get().project.samples[0].clone();
        let signal = dasp_signal::from_iter(sample.data.into_iter());
        app.handle = Some(play(signal));
    }
    if ui.button("Load sample").clicked() {
        app.store.dispatchr(Action::LoadSample { filename: "test.wav".to_string() });
    }
}
