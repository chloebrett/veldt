use super::{AsyncState, AudioState};
use crate::audio_player::play;
use crate::rpc::load_sample;
use egui::{Button, Ui};
use mesic::graph::{AmpNode, RenderableGraph};
use poll_promise::Promise;
use shared::model::Sample;
use state::{Action, Store};

pub fn sample_control(
    store: &Store,
    audio_state: &mut AudioState,
    async_state: &mut AsyncState,
    ui: &mut Ui,
) {
    if ui
        .add_enabled(
            !store.get().project.samples.is_empty(),
            Button::new("Play sample"),
        )
        .clicked()
    {
        let sample = store.get().project.samples[0].clone();
        audio_state.audio = sample.data.clone();
        let volume = store.get().volume;
        let graph = RenderableGraph::from_vec(sample.data).add_amp_node(AmpNode {
            volume,
            should_clip: true,
        });
        audio_state.handle = Some(play(graph));
    }

    if ui.button("Load sample").clicked() {
        async_state.load_sample = Some(Promise::spawn_local(async move {
            load_sample("89 BPM F# Minor.wav".to_string()).await
        }));
    }

    {
        let promise_ref: &Option<Promise<Option<Sample>>> = &async_state.load_sample;
        let mut clear = false;
        if let Some(promise) = promise_ref {
            if let Some(Some(sample)) = promise.ready() {
                clear = true;
                store.dispatchr(Action::AddSample(sample.clone()));
            }
        }
        if clear {
            async_state.load_sample = None;
        }
    }
}
