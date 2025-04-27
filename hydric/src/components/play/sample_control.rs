use crate::promise::{poll, spawn};
use crate::rpc::interleave_stereo;
use crate::rpc::load_sample;
use crate::{AsyncState, AudioState};
use egui::{Button, Ui};
use state::{Action, Store, TypeField};

pub fn sample_control(
    store: &Store,
    audio_state: &mut AudioState,
    async_state: &mut AsyncState,
    ui: &mut Ui,
) {
    if ui
        .add_enabled(
            !store.get().project.samples.is_empty(),
            Button::new("Set audio from sample"),
        )
        .clicked()
    {
        let sample = store.get().project.samples[0].clone();
        let sample = interleave_stereo(sample.left, sample.right);
        audio_state.audio = sample.clone();
        let volume = store.get().volume;
        audio_state.player.set_audio(sample, volume);
    }

    if ui.button("Load sample").clicked() {
        spawn(&mut async_state.load_sample, async move {
            load_sample("89 BPM F# Minor.wav".to_string()).await
        })
    }

    poll(&mut async_state.load_sample, |sample| {
        store.dispatchr(Action::AddChild(TypeField::Sample(sample.clone())))
    });
}
