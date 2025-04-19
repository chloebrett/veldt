use super::{
    BufferNode, CompressorNode, DelayNode, EqNode, GeneratorNode, Graph, MixerNode, ModDelayNode,
    Processor, make_graph, make_processor,
};
use crate::consts::SAMPLE_RATE;
use crate::effect::eq_filter;
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance};

/// A Graph with the required metadata to facilitate immediate processing into a Vec.
pub struct RenderGraph {
    graph: Graph,
    sample_count: usize,
    output_node_index: NodeIndex,
    processor: Processor,
    generator_indexes: Vec<NodeIndex>,

    // For iteration.
    processed_samples_count: usize,
}

impl Default for RenderGraph {
    fn default() -> Self {
        let mut graph = make_graph();
        // Set a Sum node as output to add inputs on the graph.
        let output_node_index = graph.add_node(NodeData::new2(BoxedNodeSend::new(Sum)));
        RenderGraph {
            graph,
            sample_count: 0,
            output_node_index,
            generator_indexes: vec![],
            processor: make_processor(),
            processed_samples_count: 0,
        }
    }
}

impl RenderGraph {
    // Add node with edge directed to graph output.
    pub fn add_node(&mut self, node: impl Node + 'static + Send) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        self.graph.add_edge(node_index, self.output_node_index, ());
    }

    // Add node and set as graph output.
    pub fn add_output_node(&mut self, node: impl Node + 'static + Send) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        self.graph.add_edge(self.output_node_index, node_index, ());
        // Set node as new output
        self.output_node_index = node_index;
    }

    pub fn add_generator(&mut self, node: GeneratorNode) {
        self.sample_count = usize::max(self.sample_count, node.sample_count);
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        // Keep track of node index to easily connect to Effects and Mixers
        self.generator_indexes.push(node_index);
        self.graph.add_edge(node_index, self.output_node_index, ());
    }

    pub fn add_generator_effect_with_mixer(
        &mut self,
        effect: EffectInstance,
        generator_index: usize,
    ) {
        let dry = *self
            .generator_indexes
            .get(generator_index)
            .expect("Should be a generator node at this index.");
        let mixer = self.add_effect_with_mixer(effect, dry);
        // Disconnected direct edge from generator to output.
        if let Some(edge) = self.graph.find_edge(dry, self.output_node_index) {
            self.graph.remove_edge(edge);
        };
        self.graph.add_edge(mixer, self.output_node_index, ());
    }

    pub fn add_main_effect_with_mixer(&mut self, effect: EffectInstance) {
        let dry = self.output_node_index;
        let mixer = self.add_effect_with_mixer(effect, dry);
        self.output_node_index = mixer;
    }

    fn add_effect_with_mixer(&mut self, effect: EffectInstance, dry: NodeIndex) -> NodeIndex {
        let effect_node = match effect.effect {
            Effect::SimpleEq { config } => BoxedNodeSend::new(EqNode {
                filter_left: eq_filter(&config),
                filter_right: eq_filter(&config),
            }),
            Effect::SimpleDelay { config } => {
                let delay_samples = config.delay_ms / 1000.0 * SAMPLE_RATE as f32;
                let delay_samples = delay_samples as usize;

                // Extend the graph duration by the delay amount.
                self.sample_count += delay_samples;

                BoxedNodeSend::new(DelayNode::new(delay_samples, config.feedback))
            }
            Effect::SimpleCompressor { config } => BoxedNodeSend::new(CompressorNode::new(config)),
            Effect::ModDelay { config } => BoxedNodeSend::new(ModDelayNode::new(config)),
        };
        let mixer_node = MixerNode {
            wet: effect.meta.wet,
            mute: effect.meta.mute,
        };

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
    /// Chain with .add_amp_node to control volume and/or clip.
    pub fn from_vec(vec: Vec<f32>) -> Self {
        let sample_count = vec.len();
        let buffer_node: BufferNode = vec.into();
        let mut graph = RenderGraph {
            sample_count,
            ..Default::default()
        };
        graph.add_node(buffer_node);
        graph
    }

    pub fn reset(&mut self) {
        self.processor = make_processor();
        self.processed_samples_count = 0;
    }
}

impl Iterator for RenderGraph {
    type Item = Stereo<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.processed_samples_count % Buffer::LEN == 0 {
            self.processor
                .process(&mut self.graph, self.output_node_index);
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
        AdsrEnvelope, AntiAliasingMode, DelayConfig, EffectMeta, EqConfig, EqType,
        GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, ModDelayConfig, Note,
        PitchName, PlacedNote, ScaleValue, SimpleWaveConfig, Track, TrackPlacement, WaveType,
    };

    use crate::{
        graph::{AmpNode, GeneratorNode},
        wave::{beats_to_samples, freq},
    };

    use super::*;

    fn make_track_and_track_placement() -> (Track, TrackPlacement) {
        let track = Track {
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
        };
        let track_placement = TrackPlacement {
            track_id: 0,
            offset: 0.0.into(),
            clipped_duration: None,
            visual_placement: 0,
        };
        (track, track_placement)
    }

    fn make_generator() -> GeneratorInstance {
        GeneratorInstance {
            id: 0,
            kind: GeneratorType::SimpleWave {
                config: SimpleWaveConfig {
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
                },
            },
            meta: GeneratorMeta {
                volume: 1.0,
                mute: false,
                pan: 0.0,
            },
        }
    }

    fn make_generator_node() -> GeneratorNode {
        let (track, track_placement) = make_track_and_track_placement();
        let generator = make_generator();
        let bpm = 120.0;
        let generator_node = GeneratorNode::new(generator, track, track_placement, bpm);
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
        let generator_node = make_generator_node();
        let mixer_channel = make_mixer_channel();
        let amp_node = AmpNode {
            volume: 2.3,
            should_clip: false,
        };
        let mut graph = RenderGraph::default();
        // Act
        graph.add_generator(generator_node);
        for effect in mixer_channel.effects {
            graph.add_generator_effect_with_mixer(effect, 0);
        }
        graph.add_output_node(amp_node);
        // Assert
        assert!(graph.peekable().peek().is_some())
    }

    #[test]
    fn graph_with_only_generator_renders_something() {
        // Arrange
        let generator_node = make_generator_node();
        let mut graph = RenderGraph::default();
        // Act
        graph.add_generator(generator_node);
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
        let input: Vec<f32> = (0..samples as usize)
            .map(|it| (it as f32 / SAMPLE_RATE as f32 * freq(pitch)).sin())
            .collect();
        // Act
        let graph = RenderGraph::from_vec(input.clone());
        let output: Vec<[f32; 2]> = graph.collect();
        let output_mono: Vec<f32> = output
            .iter()
            .map(|[left, right]| (left + right) * 0.5)
            .collect();
        // Assert
        assert_eq!(output_mono, input)
    }

    #[test]
    fn graph_with_clipped_duration_clips() {
        // ARRANGE
        let (track, mut track_placement) = make_track_and_track_placement();
        let generator = make_generator();
        let bpm = 120.0;
        // Set up graph with no clipping.
        let mut unclipped_graph = RenderGraph::default();
        let unclipped_generator_node = GeneratorNode::new(
            generator.clone(),
            track.clone(),
            track_placement.clone(),
            bpm,
        );
        // Set up identical graph but with a clipped duration
        let clipped_duration = 0.5;
        track_placement.clipped_duration = Some(clipped_duration.into());
        let mut clipped_graph = RenderGraph::default();
        let clipped_generator_node = GeneratorNode::new(
            generator.clone(),
            track.clone(),
            track_placement.clone(),
            bpm,
        );
        // Find what should be length of the clipped track.
        let clipped_sample_length =
            beats_to_samples(*track_placement.offset + clipped_duration, bpm) as usize;
        // ACT
        unclipped_graph.add_generator(unclipped_generator_node);
        clipped_graph.add_generator(clipped_generator_node);
        let unclipped_output: Vec<[f32; 2]> = unclipped_graph.collect();
        let clipped_output: Vec<[f32; 2]> = clipped_graph.collect();
        // ASSERT
        let expected_output: Vec<[f32; 2]> = unclipped_output[0..clipped_sample_length].to_vec();
        assert_eq!(clipped_output, expected_output)
    }

    #[test]
    fn clipped_second_track_duration_doesnt_clip_first() {
        // ARRANGE
        let (track, track_placement) = make_track_and_track_placement();
        let generator = make_generator();
        let bpm = 120.0;
        // Set two tracks' offsets so they do not overlap.
        let mut placement_1 = track_placement.clone();
        placement_1.offset = 0.0.into();
        let mut placement_2 = track_placement.clone();
        placement_2.offset = 5.0.into();
        // Create a graph with both tracks and no clipped audio
        let mut unclipped_graph = RenderGraph::default();
        let generator_node_1 =
            GeneratorNode::new(generator.clone(), track.clone(), placement_1.clone(), bpm);
        let generator_node_2 =
            GeneratorNode::new(generator.clone(), track.clone(), placement_2.clone(), bpm);
        unclipped_graph.add_generator(generator_node_1);
        unclipped_graph.add_generator(generator_node_2);
        // Set up identical graph but with a clipped duration on the second track
        let mut placement_2_clipped = placement_2.clone();
        let clipped_duration = 0.5;
        placement_2_clipped.clipped_duration = Some(clipped_duration.into());
        let mut clipped_graph = RenderGraph::default();
        let clip_generator_node_1 =
            GeneratorNode::new(generator.clone(), track.clone(), placement_1.clone(), bpm);
        let clip_generator_node_2 = GeneratorNode::new(
            generator.clone(),
            track.clone(),
            placement_2_clipped.clone(),
            bpm,
        );
        clipped_graph.add_generator(clip_generator_node_1);
        clipped_graph.add_generator(clip_generator_node_2);
        // Find what should be length of the clipped track graph output.
        let clipped_sample_length =
            beats_to_samples(*placement_2_clipped.offset + clipped_duration, bpm) as usize;
        // ACT
        // Get output of graphs
        let unclipped_output: Vec<[f32; 2]> = unclipped_graph.collect();
        let clipped_output: Vec<[f32; 2]> = clipped_graph.collect();
        // ASSERT
        let expected_output: Vec<[f32; 2]> = unclipped_output[0..clipped_sample_length].to_vec();
        assert_eq!(clipped_output, expected_output)
    }
}
