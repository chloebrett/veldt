use crate::eq::eq_filter;
use crate::eq::filter::Filter;
use crate::graph::{NoteEvent, NoteEventType, ProcessContext};
use dasp_graph::{Buffer, Input, Node};
use rand::Rng;
use shared::model::{
    EqConfig, EqType, Generator, GeneratorInstance, GeneratorMeta, NoiseConfig, NoiseType,
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
    pink_filter: Filter,
    brown_filter: Filter,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = NoiseConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            playing: false,
            pink_filter: eq_filter(&EqConfig {
                kind: EqType::SimpleFirstOrderLowPass,
                fc: 1000.0,
                q: 0.707, // See "Designing Audio Effect Plugins in C++", W. Pirkle, p273
                gain: 0.0,
            }),
            brown_filter: eq_filter(&EqConfig {
                kind: EqType::SimpleSecondOrderLowPass,
                fc: 1000.0,
                q: 0.707,
                gain: 0.0,
            }),
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
        self.state.pink_filter.apply(buffer);
    }

    fn generate_brown_noise(&mut self, buffer: &mut Buffer) {
        self.state.brown_filter.apply(buffer);
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

#[cfg(test)]
mod tests {
    use crate::node::NoiseGeneratorNode;
    use dasp_graph::Buffer;
    use shared::model::GeneratorId;
    use state::GeneratorSelector;

    const FLOAT_THRES: f32 = 1e-6;

    struct PkCoefficients {
        b0: f32,
        b1: f32,
        b2: f32,
        b3: f32,
        b4: f32,
        b5: f32,
        b6: f32,
    }

    impl PkCoefficients {
        fn pk_pink_noise(&mut self, white: f32) -> f32 {
            self.b0 = 0.99886 * self.b0 + white * 0.0555179;
            self.b1 = 0.99332 * self.b1 + white * 0.0750759;
            self.b2 = 0.96900 * self.b2 + white * 0.1538520;
            self.b3 = 0.86650 * self.b3 + white * 0.3104856;
            self.b4 = 0.55000 * self.b4 + white * 0.5329522;
            self.b5 = -0.7616 * self.b5 - white * 0.0168980;
            let pink = self.b0
                + self.b1
                + self.b2
                + self.b3
                + self.b4
                + self.b5
                + self.b6
                + white * 0.5362;
            self.b6 = white * 0.115926;

            pink
        }
    }

    fn assert_almost_equal(first: Buffer, second: Buffer) {
        if first.len() != second.len() {
            panic!("Lengths differed! {}, {}", first.len(), second.len());
        }
        for i in 0..first.len() {
            let a = first[i];
            let b = second[i];
            if (a - b).abs() > FLOAT_THRES {
                panic!("Floats {a}, {b} differed at index {i}");
            }
        }
    }

    #[test]
    #[ignore]
    fn test_pink_noise() {
        let mut pk_coeff = PkCoefficients {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
        };
        let mut node = NoiseGeneratorNode::new(GeneratorSelector(GeneratorId(0)));
        let mut rng = rand::thread_rng();
        let mut buffer = Buffer::SILENT;
        let mut pk_buffer = Buffer::SILENT;

        for i in 0..buffer.len() {
            let white = node.generate_white_noise(&mut rng);
            pk_buffer[i] = pk_coeff.pk_pink_noise(white);
        }

        node.generate_pink_noise(&mut buffer);

        assert_almost_equal(buffer, pk_buffer);
    }
}
