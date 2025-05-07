use super::pan_multipliers;
use crate::graph::ProcessContext;
use crate::wave::WaveSource;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{
    Generator, GeneratorInstance, GeneratorMeta, Placement, PlacementType, SimpleWaveConfig,
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
    sample_index: u32, // the sample that playback is currently up to.
    placements: Vec<Placement>,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            bpm: 0.0,
            wave_source: WaveSource::new(0.0),
            config: SimpleWaveConfig::default(),
            meta: GeneratorMeta::default(),
            sample_index: 0,
            placements: vec![],
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

        // Placements that are linked to this generator.
        let GeneratorSelector(generator_index) = selector;
        let placements: Vec<_> = project
            .placements
            .clone()
            .into_iter()
            .filter(|it| match &it.kind {
                PlacementType::Track(it) => it.generator_index == generator_index,
                _ => false,
            })
            .collect();
        if self.placements != placements {
            self.placements = placements;
        }

        if let Some(seek_pos) = payload.seek_pos {
            self.sample_index = seek_pos as u32;
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
    // TODO: a lot of this processing logic is generic and should be shared with
    // other generator types. How?
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
                    state.sample_index as i32 - note.global_start_sample as i32,
                ),
            );
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume_and_pan(state, out_buf, channel_index);
        }
        state.sample_index += Buffer::LEN as u32;
    }
}
