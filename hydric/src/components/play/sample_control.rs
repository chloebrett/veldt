use crate::audio_player::play;
use crate::components::{AsyncState, AudioState};
use crate::promise::{poll, spawn};
use crate::rpc::load_sample;
use egui::{Button, Ui};
use mesic::graph::{AmpNode, RenderGraph};
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
            Button::new("Play sample"),
        )
        .clicked()
    {
        let sample = store.get().project.samples[0].clone();
        audio_state.audio = sample.data.clone();
        let volume = store.get().volume;
        let mut graph = RenderGraph::from_vec(sample.data);
        graph.add_output_node(AmpNode {
            volume,
            should_clip: true,
        });
        audio_state.handle = Some(play(graph, audio_state.pre_render));
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
