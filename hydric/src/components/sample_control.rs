use super::app::App;
use crate::audio_player::play;
use crate::state::Action;
use egui::{Button, Ui};
use poll_promise::Promise;
use shared::model::Sample;
use std::rc::Rc;

pub fn sample_control(app: &mut App, ui: &mut Ui) {
    if ui
        .add_enabled(
            !app.store.get().project.samples.is_empty(),
            Button::new("Play sample"),
        )
        .clicked()
    {
        let sample = app.store.get().project.samples[0].clone();
        app.audio = sample.data.clone();
        let signal = dasp_signal::from_iter(sample.data);
        app.handle = Some(play(signal));
    }

    if ui.button("Load sample").clicked() {
        app.store.dispatchr(Action::LoadSample {
            filename: "89 BPM F# Minor.wav".to_string(),
        });
    }

    {
        let load_sample_promise: Rc<Option<Promise<Option<Sample>>>> =
            app.store.get().load_sample_promise.clone();
        let promise_ref: &Option<Promise<Option<Sample>>> = load_sample_promise.as_ref();
        if let Some(promise) = promise_ref {
            if let Some(Some(sample)) = promise.ready() {
                app.store.dispatchr(Action::ClearLoadSamplePromise);
                app.store.dispatchr(Action::AddSample(sample.clone()));
            }
        }
    }
}
