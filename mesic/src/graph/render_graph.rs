use super::ProcessContext;
use super::{
    AmpNode, BufferNode, CompressorNode, DelayNode, EqNode, Graph, MixerNode, ModDelayNode,
    Processor, SimpleWaveGeneratorNode, make_graph, make_processor,
};
use crate::consts::SAMPLE_RATE;
use crate::wave::beats_to_samples;
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::{
    Effect, EffectInstance, GeneratorInstance, GeneratorType, PlacementType, Project,
};
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

            match &project.generators[generator_index] {
                GeneratorInstance {
                    kind: GeneratorType::SimpleWave { config },
                    id: _,
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
                    self.add_simple_wave_generator(generator_node);
                }
                _ => {
                    // TODO: support adding other types of generators to the graph.
                }
            }

            // Apply effects.
            // TODO add all channels.
            // TODO: this is buggy right now! IIUC we only actually link the final effect in
            // the chain.
            let mixer_index = 0;
            let mixer_channel = &project.mixer[mixer_index];
            for (effect_index, effect) in mixer_channel.effects.iter().enumerate() {
                self.add_effect_with_mixer_to_generator(
                    mixer_index,
                    effect_index,
                    effect.clone(),
                    generator_index,
                );
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

    pub fn add_simple_wave_generator(&mut self, node: SimpleWaveGeneratorNode) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        // Keep track of node index to easily connect to Effects and Mixers
        self.generator_indexes.push(node_index);
        self.graph.add_edge(node_index, self.output_node_index, ());
    }

    pub fn add_effect_with_mixer_to_generator(
        &mut self,
        mixer_index: usize,
        effect_index: usize,
        effect: EffectInstance,
        generator_index: usize,
    ) {
        let dry = *self
            .generator_indexes
            .get(generator_index)
            .expect("Should be a generator node at this index.");
        // TODO: this isn't right. We need to add the effect to the end of the effect chain.
        // We need to make support for effect chains + multiple effect channels better.
        // Currently I think this creates a bug where only the last effect is audible.
        // Verify that.
        let mixer = self.add_effect_with_mixer(mixer_index, effect_index, effect, dry);
        // Disconnected direct edge from generator to output.
        if let Some(edge) = self.graph.find_edge(dry, self.output_node_index) {
            self.graph.remove_edge(edge);
        };
        self.graph.add_edge(mixer, self.output_node_index, ());
    }

    pub fn add_main_effect_with_mixer(
        &mut self,
        mixer_index: usize,
        effect_index: usize,
        effect: EffectInstance,
    ) {
        let dry = self.output_node_index;
        let mixer = self.add_effect_with_mixer(mixer_index, effect_index, effect, dry);
        self.output_node_index = mixer;
    }

    fn add_effect_with_mixer(
        &mut self,
        mixer_index: usize,
        effect_index: usize,
        effect: EffectInstance,
        dry: NodeIndex,
    ) -> NodeIndex {
        let effect_node = match effect.effect {
            Effect::SimpleEq { config } => {
                BoxedNodeSend::new(EqNode::new(mixer_index, effect_index, config))
            }
            Effect::SimpleDelay { config } => {
                let delay_samples = config.delay_ms / 1000.0 * SAMPLE_RATE as f32;
                let delay_samples = delay_samples as usize;

                // Extend the graph duration by the delay amount.
                self.sample_count += delay_samples;

                BoxedNodeSend::new(DelayNode::new(mixer_index, effect_index, config))
            }
            Effect::SimpleCompressor { config } => BoxedNodeSend::new(CompressorNode::new(config)),
            Effect::ModDelay { config } => BoxedNodeSend::new(ModDelayNode::new(config)),
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
        MixerChannel, ModDelayConfig, Note, PitchName, PlacedNote, Placement, ScaleValue,
        SimpleWaveConfig, Track, TrackPlacement, WaveType,
    };

    use crate::{graph::SimpleWaveGeneratorNode, wave::freq};

    use super::*;

    impl RenderGraph {
        fn set_sample_count(&mut self, count: usize) {
            self.sample_count = count;
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
                    effect: Effect::SimpleEq {
                        config: EqConfig {
                            kind: EqType::SimpleResonator,
                            fc: 1000.0,
                            q: 1.0,
                            gain: 0.0,
                        },
                    },
                    meta: EffectMeta {
                        id: 0,
                        wet: 1.0,
                        mute: false,
                    },
                },
                EffectInstance {
                    effect: Effect::SimpleDelay {
                        config: DelayConfig {
                            delay_ms: 250.0,
                            feedback: 0.5,
                        },
                    },
                    meta: EffectMeta {
                        id: 1,
                        wet: 0.5,
                        mute: false,
                    },
                },
                EffectInstance {
                    effect: Effect::ModDelay {
                        config: ModDelayConfig {
                            min_depth: 100,
                            max_depth: 200,
                            freq: 10.0,
                            lfo_type: WaveType::Triangle,
                        },
                    },
                    meta: EffectMeta {
                        id: 1,
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
        let generator_node = make_simple_wave_generator_node();
        let mixer_channel = make_mixer_channel();
        let mut graph = RenderGraph::default();
        // Act
        graph.add_simple_wave_generator(generator_node);
        for (i, effect) in mixer_channel.effects.into_iter().enumerate() {
            graph.add_effect_with_mixer_to_generator(0, i, effect, 0);
        }
        graph.add_output_amp_node();
        // Assert
        assert!(graph.peekable().peek().is_some())
    }

    #[test]
    fn graph_with_only_generator_renders_something() {
        // Arrange
        let generator_node = make_simple_wave_generator_node();
        let mut graph = RenderGraph::default();
        graph.set_sample_count(1);

        // Act
        graph.add_simple_wave_generator(generator_node);

        // Assert
        assert!(graph.peekable().peek().is_some())
    }

    #[test]
    fn graph_built_from_sample_renders_something() {
        // Arrange
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        let samples = 120;
        let input: Vec<Stereo<f32>> = (0..samples as usize)
            .map(|it| {
                let value = (it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin();
                [value, value]
            })
            .collect();

        // Act
        let graph = RenderGraph::from_vec(input.clone());
        let output: Vec<[f32; 2]> = graph.collect();

        // Assert
        assert_eq!(output, input)
    }
}
