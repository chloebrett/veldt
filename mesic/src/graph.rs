use crate::consts::SAMPLE_RATE;
use crate::effect::eq_filter;
use crate::node::{BufferNode, EqNode, MixerNode};
use dasp_graph::{BoxedNode, Buffer, Node, NodeData, node::Delay};
use petgraph::stable_graph::{NodeIndex, StableGraph};
use shared::model::{Effect, EffectInstance};
use std::iter::repeat_n;

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
    let mut vec = Vec::with_capacity(delay_samples);
    vec.extend(repeat_n(0.0, delay_samples));

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
