use crate::consts::SAMPLE_RATE;
use crate::effect::{ApplyFilter, eq_filter};
use dasp_graph::{BoxedNode, Buffer, Input, Node, NodeData, node::Delay};
use petgraph::stable_graph::{NodeIndex, StableGraph};
use shared::model::{Effect, EffectInstance};
use shared::types::{KnobPosition, Volume};
use std::cmp::min;

pub type Graph = StableGraph<NodeData<BoxedNode>, ()>;

pub type Processor = dasp_graph::Processor<Graph>;

/// A Graph with the required metadata to facilitate immediate processing into a Vec.
pub struct RenderableGraph {
    graph: Graph,
    sample_count: usize,
    output_node_index: NodeIndex,
    processor: Processor,

    // For iteration.
    processed_samples_count: usize,
}

// If these are exceeded then the graph will dynamically allocate.
const MAX_NODES: usize = 1024;
const MAX_EDGES: usize = 1024;

pub fn make_graph() -> Graph {
    Graph::with_capacity(MAX_NODES, MAX_EDGES)
}

pub fn make_processor() -> Processor {
    Processor::with_capacity(MAX_NODES)
}

impl RenderableGraph {
    pub fn new(graph: Graph, sample_count: usize, output_node_index: NodeIndex) -> Self {
        RenderableGraph {
            graph,
            sample_count,
            output_node_index,
            processor: make_processor(),
            processed_samples_count: 0,
        }
    }

    pub fn add_node(&mut self, node: impl Node + 'static) {
        let node_index = self.graph.add_node(NodeData::new1(BoxedNode::new(node)));
        self.graph.add_edge(self.output_node_index, node_index, ());
        self.output_node_index = node_index;
    }

    pub fn add_effect_with_mixer(&mut self, effect: EffectInstance) {
        let effect_node = match effect.effect {
            Effect::SimpleEq { config } => BoxedNode::new(EqNode {
                filter: eq_filter(&config),
            }),
            Effect::SimpleDelay { config } => {
                let delay_samples = config.delay_ms / 1000.0 * SAMPLE_RATE as f32;
                let delay_samples = delay_samples as usize;

                // Extend the graph duration by the delay amount.
                self.sample_count += delay_samples;

                BoxedNode::new(new_delay_node(delay_samples))
            }
            Effect::SimpleCompressor { .. } => panic!("Not implemented!"),
        };
        let mixer_node = MixerNode {
            wet: effect.meta.wet,
        };

        let dry = self.output_node_index;
        let effect = self.graph.add_node(NodeData::new1(effect_node));
        let mixer = self
            .graph
            .add_node(NodeData::new1(BoxedNode::new(mixer_node)));

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

        self.output_node_index = mixer;
    }

    /// Creates a graph that plays the buffer contained in a Vec.
    /// Chain with .add_amp_node to control volume and/or clip.
    pub fn from_vec(vec: Vec<f32>) -> Self {
        let mut graph = make_graph();
        let sample_count = vec.len();
        let buffer_node: BufferNode = vec.into();
        let buffer_node_index = graph.add_node(NodeData::new1(BoxedNode::new(buffer_node)));
        RenderableGraph::new(graph, sample_count, buffer_node_index)
    }

    pub fn reset(&mut self) {
        self.processor = make_processor();
        self.processed_samples_count = 0;
    }
}

fn new_delay_node(delay_samples: usize) -> Delay<Vec<f32>> {
    // TODO: helper function
    let mut vec = Vec::with_capacity(delay_samples);
    for _ in 0..delay_samples {
        vec.push(0.0);
    }

    Delay(vec![dasp_ring_buffer::Fixed::from(vec)])
}

impl Iterator for RenderableGraph {
    type Item = f32;

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

        // For now, only return one channel.
        let output = Some(buffers[0][self.processed_samples_count % Buffer::LEN]);

        self.processed_samples_count += 1;
        output
    }
}

/// TODO: move nodes to node.rs.

// Note containing a buffer which it outputs.
pub struct BufferNode {
    buffer: Vec<f32>,
    index: usize,
}

impl BufferNode {
    fn _reset(&mut self) {
        self.index = 0;
    }
}

impl From<Vec<f32>> for BufferNode {
    fn from(item: Vec<f32>) -> Self {
        BufferNode {
            buffer: item,
            index: 0,
        }
    }
}

impl Node for BufferNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        for out_buf in output {
            let start_index = self.index;
            let end_index = min(start_index + Buffer::LEN, self.buffer.len());

            let mut slice = if start_index < end_index {
                &self.buffer[start_index..end_index]
            } else {
                &vec![]
            };

            // If the slice isn't long enough, fill the rest with zeroes.
            let mut vec;
            if slice.len() < Buffer::LEN {
                vec = slice.to_vec();
                vec.resize(Buffer::LEN, 0.0);
                slice = &vec;
            }

            if slice.len() > Buffer::LEN {
                panic!(
                    "Got a slice with length {}, which is more than {} and shouldn't happen!",
                    slice.len(),
                    Buffer::LEN
                );
            }

            out_buf.copy_from_slice(slice);
        }
        self.index += Buffer::LEN;
    }
}

pub struct EqNode {
    pub filter: Box<dyn ApplyFilter>,
}

impl Node for EqNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            let buf = self.filter.apply(in_buf);
            out_buf.copy_from_slice(&buf);
        }
    }
}

/// Node with a volume control.
/// Can clip the post-gain signal if desired.
pub struct AmpNode {
    pub volume: Volume,
    pub should_clip: bool,
}

impl Node for AmpNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            let buf: Vec<f32> = in_buf
                .iter()
                .map(|it| {
                    let mut amped = it * self.volume;
                    if self.should_clip {
                        amped = amped.clamp(-1.0, 1.0);
                    }
                    amped
                })
                .collect();
            out_buf.copy_from_slice(&buf);
        }
    }
}

/// Mixes two inputs down to one in the given wet/dry ratio.
/// The first input is the dry signal. The second is the wet signal.
/// wet = 1.0 returns only the wet signal.
/// wet = 0.0 returns only the dry signal.
/// wet = 0.5 returns a 50/50 mix.
/// And so on.
pub struct MixerNode {
    pub wet: KnobPosition,
}

impl Node for MixerNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        debug_assert!(self.wet >= 0.0 && self.wet <= 1.0);
        let dry = 1.0 - self.wet;

        for ((out_buf, dry_buf), wet_buf) in output
            .iter_mut()
            .zip(
                inputs
                    .first()
                    .expect("Expected a dry signal as the first input")
                    .buffers(),
            )
            .zip(
                inputs
                    .get(1)
                    .expect("Expected a wet signal as the second input")
                    .buffers(),
            )
        {
            let buf: Vec<f32> = dry_buf
                .iter()
                .zip(wet_buf.iter())
                .map(|(d, w)| d * dry + w * self.wet)
                .collect();
            out_buf.copy_from_slice(&buf);
        }
    }
}
