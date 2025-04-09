use super::{
    BufferNode, CompressorNode, EqNode, Graph, MixerNode, Processor, make_graph, make_processor,
};
use crate::consts::SAMPLE_RATE;
use crate::effect::eq_filter;
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Delay};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance};

/// A Graph with the required metadata to facilitate immediate processing into a Vec.
pub struct RenderGraph {
    graph: Graph,
    sample_count: usize,
    output_node_index: NodeIndex,
    processor: Processor,

    // For iteration.
    processed_samples_count: usize,
}

impl RenderGraph {
    pub fn new(graph: Graph, sample_count: usize, output_node_index: NodeIndex) -> Self {
        RenderGraph {
            graph,
            sample_count,
            output_node_index,
            processor: make_processor(),
            processed_samples_count: 0,
        }
    }

    pub fn add_node(&mut self, node: impl Node + 'static + Send) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        self.graph.add_edge(self.output_node_index, node_index, ());
        self.output_node_index = node_index;
    }

    pub fn add_effect_with_mixer(&mut self, effect: EffectInstance) {
        let effect_node = match effect.effect {
            Effect::SimpleEq { config } => BoxedNodeSend::new(EqNode {
                filter: eq_filter(&config),
            }),
            Effect::SimpleDelay { config } => {
                let delay_samples = config.delay_ms / 1000.0 * SAMPLE_RATE as f32;
                let delay_samples = delay_samples as usize;

                // Extend the graph duration by the delay amount.
                self.sample_count += delay_samples;

                BoxedNodeSend::new(new_delay_node(delay_samples))
            }
            Effect::SimpleCompressor { config } => BoxedNodeSend::new(CompressorNode::new(config)),
        };
        let mixer_node = MixerNode {
            wet: effect.meta.wet,
            mute: effect.meta.mute,
        };

        let dry = self.output_node_index;
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

        self.output_node_index = mixer;
    }

    /// Creates a graph that plays the buffer contained in a Vec.
    /// Chain with .add_amp_node to control volume and/or clip.
    pub fn from_vec(vec: Vec<f32>) -> Self {
        let mut graph = make_graph();
        let sample_count = vec.len();
        let buffer_node: BufferNode = vec.into();
        let buffer_node_index = graph.add_node(NodeData::new2(BoxedNodeSend::new(buffer_node)));
        RenderGraph::new(graph, sample_count, buffer_node_index)
    }

    pub fn reset(&mut self) {
        self.processor = make_processor();
        self.processed_samples_count = 0;
    }
}

/// The delay node is built into dasp_graph. This is just a helper to construct one.
fn new_delay_node(delay_samples: usize) -> Delay<Vec<f32>> {
    Delay(vec![dasp_ring_buffer::Fixed::from(vec![
        0.0;
        delay_samples
    ])])
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
