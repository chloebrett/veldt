use crate::eq::pink::PinkFilter;
use crate::graph::{NoteEvent, NoteEventType, ProcessContext};
use dasp_graph::{Buffer, Input, Node};
use rand::Rng;
use shared::model::{
    Generator, GeneratorInstance, GeneratorMeta, NoiseConfig, NoiseType,
};
use state::GeneratorSelector;

pub struct NoiseGeneratorNode {
    selector: GeneratorSelector,
    state: NodeState,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    config: NoiseConfig,
    meta: GeneratorMeta,
    playing: bool,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = NoiseConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            playing: false,
        }
    }
}

impl NodeState {
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        if let GeneratorInstance {
            it: Generator::Noise(config),
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

    fn apply_volume(state: &NodeState, buffer: &mut Buffer) {
        for x in buffer.iter_mut() {
            *x *= state.meta.volume;
        }
    }

    fn generate_white_noise(&self, rng: &mut impl Rng) -> f32 {
        let min = -1.0;
        let max = 1.0;
        generate_random_number_in_range(rng, min, max)
    }

    fn generate_pink_noise(&mut self, buffer: &mut Buffer) {
        let mut filter = PinkFilter::new();
        filter.apply(buffer);
    }

    fn generate_brown_noise(&mut self, buffer: &mut Buffer) {
        for _ in 0..2 {
            let mut filter = PinkFilter::new();
            filter.apply(buffer);
        }
    }
}

impl Node<ProcessContext> for NoiseGeneratorNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let kind = self.state.config.kind;
        self.state.update(payload, self.selector);

        let mut buffer = Buffer::SILENT;
        let mut rng = rand::thread_rng();
        let GeneratorSelector(generator_id) = self.selector;

        if payload.stop_generators.get(&generator_id) == Some(&true) {
            log::info!("Stopped noise: {:?}", generator_id);
            self.state.playing = false;
        }

        for i in 0..buffer.len() {
            let mut events: Vec<NoteEvent> = payload
                .note_events
                .get(&generator_id)
                .cloned()
                .unwrap_or(vec![])
                .into_iter()
                .filter(|it| it.sample_index == i)
                .collect();

            // Special case: if there are both note_on and note_off events in a single sample,
            // don't process the note_off events.
            if events.iter().any(|it| it.kind == NoteEventType::On) {
                events.retain(|it| it.kind == NoteEventType::On);
            }

            for note_event in events {
                match note_event.kind {
                    NoteEventType::On => self.state.playing = true,
                    NoteEventType::Off => self.state.playing = false,
                }
            }

            if self.state.playing {
                buffer[i] = self.generate_white_noise(&mut rng);
            }
        }

        match kind {
            NoiseType::White => {}
            NoiseType::Pink => self.generate_pink_noise(&mut buffer),
            NoiseType::Brown => self.generate_brown_noise(&mut buffer),
        }

        for out_buf in output.iter_mut() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume(&self.state, out_buf);
        }
    }
}

// Generates random number between [min, max]
pub fn generate_random_number_in_range(rng: &mut impl Rng, min: f32, max: f32) -> f32 {
    rng.gen_range(min..=max)
}
