use crate::effect::ApplyEffect;
use dasp_graph::{BoxedNode, Buffer, Input, Node, NodeData};
use petgraph::stable_graph::{NodeIndex, StableGraph};
use shared::model::Effect;
use shared::model::EffectInstance;
use shared::types::Volume;
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
    processed_samples_count: usize, // index within buffer.
    processed_buffers_count: usize, // number of buffers processed.
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
            processed_samples_count: Buffer::LEN, // to force a first render.
            processed_buffers_count: 0,
        }
    }

    pub fn add_amp_node(mut self, amp_node: AmpNode) -> Self {
        let amp_node_index = self
            .graph
            .add_node(NodeData::new1(BoxedNode::new(amp_node)));
        self.graph
            .add_edge(self.output_node_index, amp_node_index, ());
        self.output_node_index = amp_node_index;
        self
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
        self.processed_buffers_count = 0;
    }
}

impl Iterator for RenderableGraph {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.processed_samples_count >= Buffer::LEN {
            self.processor
                .process(&mut self.graph, self.output_node_index);
            self.processed_samples_count = 0;
            self.processed_buffers_count += 1;
        }

        if self.processed_buffers_count * Buffer::LEN + self.processed_samples_count
            >= self.sample_count
        {
            return None;
        }

        let buffers = &self
            .graph
            .node_weight(self.output_node_index)
            .unwrap()
            .buffers;

        // For now, only return one channel.
        let output = Some(buffers[0][self.processed_samples_count]);

        self.processed_samples_count += 1;
        output
    }
}

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
            let index = self.index;
            let mut slice = &self.buffer[index..min(index + Buffer::LEN, self.buffer.len())];

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

pub struct EffectNode {
    instance: EffectInstance,
}

impl Node for EffectNode {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for (out_buf, in_buf) in output
            .iter_mut()
            .zip(inputs.first().expect("Expected one input").buffers())
        {
            let buf = match &self.instance.effect {
                // TODO: use a dasp_graph Delay node.
                Effect::SimpleDelay { config } => &config.apply(in_buf),
                // TODO: store state on the EQ nodes, so that they don't lose track
                // of their state every 64 samples.
                Effect::SimpleEq { config } => &config.apply(in_buf),
                _ => panic!("Effect not implemented yet!"),
            };
            out_buf.copy_from_slice(buf);
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
