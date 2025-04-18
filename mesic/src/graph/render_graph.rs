use super::{
    BufferNode, CompressorNode, DelayNode, EqNode, Graph, MixerNode, ModDelayNode, Processor,
    make_graph, make_processor,
};
use crate::consts::SAMPLE_RATE;
use crate::effect::eq_filter;
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::SumBuffers};
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
        // Set a Buffer Summer as output all inputs on the graph.
        let output_node_index = graph.add_node(NodeData::new2(BoxedNodeSend::new(SumBuffers {})));
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
    pub fn add_generator(&mut self, node: impl Node + 'static + Send, sample_count: usize) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        // Keep track of node index to easily connect to Effects and Mixers
        self.generator_indexes.push(node_index);
        self.sample_count = usize::max(self.sample_count, sample_count);
        self.graph.add_edge(node_index, self.output_node_index, ());
    }

    pub fn add_node(&mut self, node: impl Node + 'static + Send) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        self.graph.add_edge(node_index, self.output_node_index, ());
    }

    pub fn add_output_node(&mut self, node: impl Node + 'static + Send) {
        let node_index = self
            .graph
            .add_node(NodeData::new2(BoxedNodeSend::new(node)));
        self.graph.add_edge(self.output_node_index, node_index, ());
        // Set node as new output
        self.output_node_index = node_index
    }

    pub fn add_effect_with_mixer(&mut self, effect: EffectInstance, generator_index: usize) {
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

        let dry = *self
            .generator_indexes
            .get(generator_index)
            .expect("Should have been a generator on the graph with this index.");
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

        // Conect to output summer
        self.graph.add_edge(mixer, self.output_node_index, ());
        if let Some(edge) = self.graph.find_edge(dry, self.output_node_index) {
            self.graph.remove_edge(edge);
        };
    }

    /// Creates a graph that plays the buffer contained in a Vec.
    /// Chain with .add_amp_node to control volume and/or clip.
    pub fn from_vec(vec: Vec<f32>) -> Self {
        let sample_count = vec.len();
        let buffer_node: BufferNode = vec.into();
        let mut graph = RenderGraph::default();
        graph.sample_count = sample_count;
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
