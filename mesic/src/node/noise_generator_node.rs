use crate::graph::{NoteEventType, ProcessContext};
use dasp_graph::{Buffer, Input, Node};
use rand::Rng;
use shared::model::{Generator, GeneratorInstance, GeneratorMeta, NoiseConfig, NoiseType};
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
    pink_b: [f32; 7],  // Coefficients for pink noise
    last_brown_output: f32,  // Last output for brown noise
}

impl Default for NodeState {
    fn default() -> Self {
        let config = NoiseConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            playing: false,
            pink_b: [0.0; 7],
            last_brown_output: 0.0,
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

    fn generate_white_noise(rng: &mut impl Rng) -> f32 {
        let min = -1.0;
        let max = 1.0;
        generate_random_number_in_range(rng, min, max)
    }

    fn generate_brown_noise(&mut self, rng: &mut impl Rng) -> f32 {
        let wt = Self::generate_white_noise(rng);
        let leak = 0.02;
        let mut output = (self.state.last_brown_output + wt * leak).clamp(-1.0, 1.0);
        output *= 0.95; // Optional, reduces popping at the end
        self.state.last_brown_output = output;
        output
    }

    fn generate_pink_noise(&mut self, rng: &mut impl Rng) -> f32 {
        // From Paul Kellet's implementation: https://www.musicdsp.org/en/latest/Filters/76-pink-noise-filter.html
        let wt = Self::generate_white_noise(rng);

        self.state.pink_b[0] = 0.99886 * self.state.pink_b[0] + wt * 0.0555179;
        self.state.pink_b[1] = 0.99332 * self.state.pink_b[1] + wt * 0.0750759;
        self.state.pink_b[2] = 0.96900 * self.state.pink_b[2] + wt * 0.1538520;
        self.state.pink_b[3] = 0.86650 * self.state.pink_b[3] + wt * 0.3104856;
        self.state.pink_b[4] = 0.55000 * self.state.pink_b[4] + wt * 0.5329522;
        self.state.pink_b[5] = -0.7616 * self.state.pink_b[5] - wt * 0.0168980;
        let pink = self.state.pink_b[0] + self.state.pink_b[1] + self.state.pink_b[2] + self.state.pink_b[3] + self.state.pink_b[4] + self.state.pink_b[5] + self.state.pink_b[6] + wt * 0.5362;
        self.state.pink_b[6] = wt * 0.115926;
        (pink * 0.11).clamp(-1.0, 1.0)
    }

    fn generate_noise_sample(&mut self, rng: &mut impl Rng, kind: NoiseType) -> f32 {
        match kind {
            NoiseType::White => Self::generate_white_noise(rng),
            NoiseType::Pink => self.generate_pink_noise(rng),
            NoiseType::Brown => self.generate_brown_noise(rng),
        }
    }
}

impl Node<ProcessContext> for NoiseGeneratorNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let kind = self.state.config.kind;
        self.state.update(payload, self.selector);

        let mut buffer = Buffer::SILENT;
        let mut rng = rand::thread_rng();
        let GeneratorSelector(generator_index) = self.selector;

        for i in 0..buffer.len() {
            let mut events: Vec<_> = payload.note_events[generator_index]
                .clone()
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
                buffer[i] = self.generate_noise_sample(&mut rng, kind);
            }
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