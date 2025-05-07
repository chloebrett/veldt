use super::pan_multipliers;
use crate::graph::ProcessContext;
use crate::wave::WaveSource;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{
    Generator, GeneratorInstance, GeneratorMeta, SimpleWaveConfig,
};
use shared::types::Beats;
use state::GeneratorSelector;

pub struct SimpleWaveGeneratorNode {
    selector: GeneratorSelector,
    state: NodeState,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    bpm: Beats,
    wave_source: WaveSource,
    config: SimpleWaveConfig,
    meta: GeneratorMeta,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            bpm: 0.0,
            wave_source: WaveSource::new(0.0),
            config: SimpleWaveConfig::default(),
            meta: GeneratorMeta::default(),
        }
    }
}

impl NodeState {
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        let project = &payload.store.project;
        let bpm = project.bpm;

        if bpm != self.bpm {
            self.bpm = bpm;
            self.wave_source = WaveSource::new(bpm);
        }

        if let GeneratorInstance {
            it: Generator::SimpleWave(config),
            meta,
            ..
        } = &payload.store.select(&selector)
        {
            if self.config != *config {
                self.config = config.clone();
            }
            if self.meta != *meta {
                self.meta = meta.clone();
            }
        }
    }
}

impl SimpleWaveGeneratorNode {
    pub fn new(selector: GeneratorSelector) -> Self {
        Self {
            selector,
            state: NodeState::default(),
        }
    }

    fn apply_volume_and_pan(state: &NodeState, buffer: &mut Buffer, channel_index: usize) {
        let pan_mult = pan_multipliers(state.meta.pan)[channel_index];
        for x in buffer.iter_mut() {
            *x *= pan_mult * state.meta.volume;
        }
    }
}

impl Node<ProcessContext> for SimpleWaveGeneratorNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let state = &mut self.state;
        state.update(payload, self.selector);

        // Skip generating if muted!
        // TODO: disconnect muted generators from the graph.
        // This should be handled from the mixer.
        if state.meta.mute || state.meta.volume == 0.0 {
            return;
        }

        let mut buffer = Buffer::SILENT;
        let GeneratorSelector(generator_index) = self.selector;

        for note in &payload.notes[generator_index] {
            dasp_slice::add_in_place(
                &mut buffer,
                &state.wave_source.unison_wave(
                    note.pitch_name.into(),
                    note.duration,
                    &state.config,
                    note.samples_since_started,
                ),
            );
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume_and_pan(state, out_buf, channel_index);
        }
    }
}
