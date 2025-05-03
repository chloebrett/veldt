use super::ProcessContext;
use super::{
    AmpNode, BufferNode, CompressorNode, DelayNode, EqNode, Graph, MixerNode, ModDelayNode,
    Processor, SimpleWaveGeneratorNode, make_graph, make_processor,
};
use crate::consts::SAMPLE_RATE;
use crate::graph::subsynth_node::SubSynthNode;
use crate::wave::beats_to_samples;
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance, Generator, GeneratorInstance, PlacementType, Project};
use state::{Action, Selector};
use std::sync::mpsc::Receiver;

/// A Graph with the required metadata to facilitate immediate processing into a Vec.
pub struct RenderGraph {
    graph: Graph,
    sample_count: usize,
    output_node_index: NodeIndex,
    processor: Processor,
    generator_indexes: Vec<NodeIndex>,

    // Contains a copy of the project.
    // Updated based on actions from the main store at each buffer cycle.
    process_context: ProcessContext,
    // Receives actions from the main store and applies to mesic store.
    rx: Option<Receiver<(Selector, Action)>>,

    // For iteration.
    processed_samples_count: usize,
}

impl Default for RenderGraph {
    fn default() -> Self {
        let mut graph = make_graph();

        // There is always an output node, and it's usually either a SumNode
        // or an AmpNode. We start with a Sum node, then can later add an AmpNode after it.
        // The SumNode is the destination for all the generator/effect instance chains -
        // they all get summed together to produce the final audio.
        // Note: consider adding the starter AmpNode here as well.
        let output_node_index = graph.add_node(NodeData::new2(BoxedNodeSend::new(Sum)));

        Self {
            graph,
            sample_count: 0,
            output_node_index,
            generator_indexes: vec![],
            processor: make_processor(),
            process_context: ProcessContext::default(),
            rx: None,
            processed_samples_count: 0,
        }
    }
}

impl RenderGraph {
    /// Deletes all nodes from the graph.
    pub fn clear_nodes(&mut self) {
        self.graph = make_graph();

        // Add a Sum node for the same reason as in default().
        self.output_node_index = self.graph.add_node(NodeData::new2(BoxedNodeSend::new(Sum)));

        // Reset counters.
        self.sample_count = 0;
        self.generator_indexes = vec![];
        self.processor = make_processor();
        self.processed_samples_count = 0;

        // Keep the process context and rx because they contain the store.
    }

    pub fn set_from_audio(&mut self, audio: Vec<Stereo<f32>>) {
        self.sample_count = audio.len();
        let buffer_node: BufferNode = audio.into();
        self.add_pre_output_node(buffer_node);
        self.add_output_amp_node();
    }

    /// Initializes the graph from a project instance.
    /// Not idempotent! Only call this on a fresh RenderGraph. (either new or call clear_nodes).
    /// This is mostly an interim method until we get action receiving working properly.
    pub fn set_from_project(&mut self, project: &Project) {
        let bpm = project.bpm;

        for generator_index in 0..project.generators.len() {
            // Placements that are linked to this generator.
            let placements: Vec<_> = project
                .placements
                .clone()
                .into_iter()
                .filter(|it| match &it.kind {
                    PlacementType::Track(it) => it.generator_index == generator_index,
                    _ => false,
                })
                .collect();

            // TODO: do better than just cloning all the tracks!
            // Perhaps load the relevant track data from the store
            // out of the payload in each processing cycle?
            // Or even just get a &[Track] containing all the tracks in the store.
            let tracks = project.tracks.clone();

            let generator_node_index = match &project.generators[generator_index] {
                GeneratorInstance {
                    it: Generator::SimpleWave(config),
                    meta,
                } => {
                    let generator_node = SimpleWaveGeneratorNode::new(
                        config.clone(),
                        meta.clone(),
                        generator_index,
                        placements,
                        tracks,
                        bpm,
                    );
                    self.add_simple_wave_generator(generator_node)
                }
                GeneratorInstance {
                    it: Generator::SubSynth(config),
                    meta,
                } => {
                    let generator_node = SubSynthNode::new(
                        config.clone(),
                        meta.clone(),
                        generator_index,
                        placements,
                        tracks,
                        bpm,
                    );
                    self.add_subsynth_generator(generator_node)
                }
                _ => {
                    // TODO: support adding other types of generators to the graph.
                    panic!("Not yet implemented.")
                }
            };

            // Apply effects.
            // TODO add all channels.
            if project.mixer.is_empty() {
                continue;
            }
            let mixer_index = 0;
            let mixer_channel = &project.mixer[mixer_index];
            let mut prev_node = generator_node_index;
            // Add effects in a chain inputting the output of one to the next.
            for (effect_index, effect) in mixer_channel.effects.iter().enumerate() {
                let next_node = self.add_effect_with_mixer(
                    mixer_index,
                    effect_index,
                    effect.clone(),
                    prev_node,
                );
                prev_node = next_node;
            }
            if prev_node != generator_node_index {
                // Effects have been applied.
                // Remove edge between generator and output
                if let Some(edge) = self
                    .graph
                    .find_edge(generator_node_index, self.output_node_index)
                {
                    self.graph.remove_edge(edge);
                };
                // Add edge between final mixer and output.
                self.graph.add_edge(prev_node, self.output_node_index, ());
            }
        }

        self.add_output_amp_node();
        self.sample_count = beats_to_samples(*project.duration(), bpm) as usize;
    }

    pub fn pos(&self) -> usize {
        self.processed_samples_count
    }

    pub fn seek(&mut self, samples: usize) {
        self.processed_samples_count = samples;
        self.process_context.seek_pos = Some(samples);
    }

    pub fn set_receiver(&mut self, receiver: Receiver<(Selector, Action)>) {
        self.rx = Some(receiver);
    }

    fn add_node(&mut self, node: impl Node<ProcessContext> + 'static + Send) -> NodeIndex {
        self.graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)))
    }

    // Adds a node just before the current output.
    pub fn add_pre_output_node(&mut self, node: impl Node<ProcessContext> + 'static + Send) {
        let node_index = self.add_node(node);
        self.graph.add_edge(node_index, self.output_node_index, ());
    }

    // Add amp node and set as graph output.
    pub fn add_output_amp_node(&mut self) {
        let node_index = self.add_node(AmpNode::default());
        self.graph.add_edge(self.output_node_index, node_index, ());
        // Set node as new output
        self.output_node_index = node_index;
    }

    pub fn add_simple_wave_generator(&mut self, node: SimpleWaveGeneratorNode) -> NodeIndex {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        // Keep track of node index to easily connect to Effects and Mixers
        self.generator_indexes.push(node_index);
        self.graph.add_edge(node_index, self.output_node_index, ());
        node_index
    }

    pub fn add_subsynth_generator(&mut self, node: SubSynthNode) -> NodeIndex {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        // Keep track of node index to easily connect to Effects and Mixers
        self.generator_indexes.push(node_index);
        self.graph.add_edge(node_index, self.output_node_index, ());
        node_index
    }

    // Adds an effect with a mixer to the graph.
    fn add_effect_with_mixer(
        &mut self,
        mixer_index: usize,
        effect_index: usize,
        effect: EffectInstance,
        dry: NodeIndex,
    ) -> NodeIndex {
        let effect_node = match effect.it {
            Effect::SimpleEq(config) => {
                BoxedNodeSend::new(EqNode::new(mixer_index, effect_index, config))
            }
            Effect::Delay(config) => {
                let delay_samples = config.delay_ms / 1000.0 * SAMPLE_RATE as f32;
                let delay_samples = delay_samples as usize;

                // Extend the graph duration by the delay amount.
                self.sample_count += delay_samples;

                BoxedNodeSend::new(DelayNode::new(mixer_index, effect_index, config))
            }
            Effect::Compressor(config) => BoxedNodeSend::new(CompressorNode::new(config)),
            Effect::ModDelay(config) => BoxedNodeSend::new(ModDelayNode::new(config)),
        };
        let mixer_node = MixerNode::new(mixer_index, effect_index, effect.meta);

        let effect = self.graph.add_node(NodeData::new2(effect_node));
        let mixer = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(mixer_node)));

        // Route the signal like this:
        // dry --|
        //  |    |
        //  v    |
        // eff   |
        //  |    |
        //  v    |
        // mix <-|
        //  |
        //  v
        self.graph.add_edge(dry, effect, ());
        // Note that dry goes second. If the two lines below are switched, the mixer `wet` is
        // inverted!
        self.graph.add_edge(effect, mixer, ());
        self.graph.add_edge(dry, mixer, ());
        mixer
    }

    /// Adds an effect node with a mixer directly to output of graph.
    pub fn add_main_effect_with_mixer(
        &mut self,
        mixer_index: usize,
        effect_index: usize,
        effect: EffectInstance,
    ) {
        self.output_node_index =
            self.add_effect_with_mixer(mixer_index, effect_index, effect, self.output_node_index);
    }

    /// Creates a graph that plays the buffer contained in a Vec.
    pub fn from_vec(vec: Vec<Stereo<f32>>) -> Self {
        let mut graph = Self::default();
        graph.set_from_audio(vec);
        graph
    }

    fn update_store(&mut self) {
        // Update the store if there are actions to process.
        let store = &mut self.process_context.store;
        if let Some(rx) = &self.rx {
            while let Ok((selector, action)) = rx.try_recv() {
                // TODO: also update graph topology by listening for the appropriate actions.
                // E.g. add/remove effect or generator.
                store.update(&selector, &action);
            }
        }
    }
}

impl Iterator for RenderGraph {
    type Item = Stereo<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_store();

        if self.processed_samples_count % Buffer::LEN == 0 {
            self.processor.process(
                &mut self.graph,
                &self.process_context,
                self.output_node_index,
            );
            self.process_context.seek_pos = None;
        }

        if self.processed_samples_count >= self.sample_count {
            return None;
        }

        let buffers = &self
            .graph
            .node_weight(self.output_node_index)
            .unwrap()
            .buffers;

        let left = buffers[0][self.processed_samples_count % Buffer::LEN];
        let right = buffers[1][self.processed_samples_count % Buffer::LEN];
        let output = Some([left, right]);

        self.processed_samples_count += 1;
        output
    }

    // TODO: implement size_hint or SizedIterator to make collection more efficient.
}

#[cfg(test)]
mod tests {
    use shared::model::{
        AdsrEnvelope, AntiAliasingMode, DelayConfig, EffectMeta, EqConfig, EqType, GeneratorMeta,
        MixerChannel, ModDelayConfig, ModMatrix, Note, PitchName, PlacedNote, Placement,
        ScaleValue, SimpleWaveConfig, Track, TrackPlacement, WaveType,
    };

    use crate::{graph::SimpleWaveGeneratorNode};
    use shared::types::Freq;

    use super::*;

    impl RenderGraph {
        fn set_sample_count(&mut self, count: usize) {
            self.sample_count = count;
        }
    }

    // Root Mean Squared to calculate if there is signal in output.
    fn rms(graph: RenderGraph) -> f32 {
        let mut left_sum = 0.0;
        let mut right_sum = 0.0;
        let mut count = 0;
        for [left, right] in graph {
            count += 1;
            left_sum += left.powi(2);
            right_sum += right.powi(2);
        }
        ((left_sum / count as f32 + right_sum / count as f32) / 2.0).sqrt()
    }

    fn make_project() -> Project {
        Project {
            name: "test".into(),
            tracks: vec![make_track()],
            placements: vec![make_placement()],
            samples: vec![],
            generators: vec![GeneratorInstance {
                it: Generator::SimpleWave(make_simple_wave_config()),
                meta: make_generator_meta(),
            }],
            mixer: vec![make_mixer_channel()],
            bpm: 120.0,
            mod_matrix: ModMatrix::default(),
        }
    }

    fn make_track() -> Track {
        Track {
            notes: vec![PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: ScaleValue::C,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: 0.0.into(),
            }],
            offset: 0.0.into(),
        }
    }

    fn make_placement() -> Placement {
        Placement {
            kind: PlacementType::Track(TrackPlacement {
                track_index: 0,
                generator_index: 0,
            }),
            offset: 0.0.into(),
            clipped_duration: None,
            visual_placement: 0,
        }
    }

    fn make_simple_wave_config() -> SimpleWaveConfig {
        SimpleWaveConfig {
            wave: WaveType::Sine,
            envelope: AdsrEnvelope {
                attack: 0.1,
                decay: 0.1,
                sustain: 0.8,
                release: 0.1,
            },
            osc_count: 4,
            detune_cents: 5.0,
            anti_aliasing_mode: AntiAliasingMode::Off,
            oversample_factor: 2,
        }
    }

    fn make_generator_meta() -> GeneratorMeta {
        GeneratorMeta {
            volume: 1.0,
            mute: false,
            pan: 0.0,
        }
    }

    fn make_simple_wave_generator_node() -> SimpleWaveGeneratorNode {
        let track = make_track();
        let placement = make_placement();
        let bpm = 120.0;
        let generator_node = SimpleWaveGeneratorNode::new(
            make_simple_wave_config(),
            make_generator_meta(),
            0,
            vec![placement],
            vec![track],
            bpm,
        );
        generator_node
    }

    fn make_mixer_channel() -> MixerChannel {
        MixerChannel {
            effects: vec![
                EffectInstance {
                    it: Effect::SimpleEq(EqConfig {
                        kind: EqType::SimpleResonator,
                        fc: 1000.0,
                        q: 1.0,
                        gain: 0.0,
                    }),
                    meta: EffectMeta {
                        wet: 1.0,
                        mute: false,
                    },
                },
                EffectInstance {
                    it: Effect::Delay(DelayConfig {
                        delay_ms: 250.0,
                        feedback: 0.5,
                    }),
                    meta: EffectMeta {
                        wet: 0.5,
                        mute: false,
                    },
                },
                EffectInstance {
                    it: Effect::ModDelay(ModDelayConfig {
                        min_depth: 100,
                        max_depth: 200,
                        freq: 10.0,
                        lfo_type: WaveType::Triangle,
                    }),
                    meta: EffectMeta {
                        wet: 0.5,
                        mute: false,
                    },
                },
            ],
        }
    }

    #[test]
    fn empty_render_graph_renders_nothing() {
        let graph = RenderGraph::default();
        // Iterator should be empty.
        let output: Vec<[f32; 2]> = graph.collect();
        assert!(output.is_empty())
    }

    #[test]
    fn basic_render_graph_renders_something() {
        // Arrange
        let mut graph = RenderGraph::default();
        graph.set_from_project(&make_project());
        // Act
        graph.add_output_amp_node();
        // Assert
        assert!(rms(graph) > 0.0)
    }

    #[test]
    fn graph_with_only_generator_renders_something() {
        // Arrange
        let mut project = make_project();
        project.mixer = vec![];
        let mut graph = RenderGraph::default();

        // Act
        graph.set_from_project(&project);

        // Assert
        assert!(rms(graph) > 0.0);
    }

    #[test]
    fn graph_built_from_sample_renders_correctly() {
        // Arrange
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        let samples = 120;
        let input: Vec<Stereo<f32>> = (0..samples as usize)
            .map(|it| {
                let freq: Freq = pitch.into();
                let value = (it as f32 / SAMPLE_RATE as f32 * freq).sin();
                [value, value]
            })
            .collect();

        // Act
        let graph = RenderGraph::from_vec(input.clone());
        let output: Vec<[f32; 2]> = graph.collect();

        // Assert
        assert_eq!(output, input)
    }

    #[test]
    fn adding_effects_changes_output() {
        // Arrange
        let mut project = make_project();
        let mut graph = RenderGraph::default();
        let mut graph_no_mixer = RenderGraph::default();

        // Act
        graph.set_from_project(&project);
        project.mixer = vec![];
        graph_no_mixer.set_from_project(&project);

        // Assert
        assert_ne!(
            graph.collect::<Vec<[f32; 2]>>(),
            graph_no_mixer.collect::<Vec<[f32; 2]>>()
        );
    }
}
