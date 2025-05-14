use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::envelope::EnvelopeGenerator;
use crate::graph::{NoteEventType, ProcessContext};
use crate::maths::linspace;
use crate::wave::detune_multiplier;
use crate::wave_cache::{WaveCache, WaveKey};
use crate::rng::generate_white_noise;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Generator, GeneratorInstance, GeneratorMeta, PitchName, NoiseConfig};
use shared::types::Freq;
use state::GeneratorSelector;

pub struct NoiseGeneratorNode {
    selector: GeneratorSelector,
    state: NodeState,
    cache: WaveCache,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    config: NoiseConfig,
    meta: GeneratorMeta,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = NoiseConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
        }
    }
}

impl NodeState {
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        if let GeneratorInstance {
            it: Generator::NoiseConfig(config),
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

impl NoiseGeneratorNode {
    pub fn new(selector: GeneratorSelector) -> Self {
        Self {
            selector,
            state: NodeState::default(),
        }
    }

    fn apply_volume(state: &NodeState, buffer: &mut Buffer, channel_index: usize) {
        for x in buffer.iter_mut() {
            *x *= state.meta.volume;
        }
    }

    fn generate_noise_sample(&self) -> f64 {
        match self.state.config.kind {
            NoiseType::White => generate_white_noise(),
            NoiseType::Pink => todo!(),
            NoiseType::Brown => todo!(),
        }
    }
}

impl Node<ProcessContext> for NoiseGeneratorNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let state = &mut self.state;
        state.update(payload, self.selector);

        let mut buffer = Buffer::SILENT;
        let GeneratorSelector(generator_index) = self.selector;

        for i in 0..buffer.len() {
            buffer[i] = self.generate_noise_sample();
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume(state, out_buf, channel_index);
        }
    }
}