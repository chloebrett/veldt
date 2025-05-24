use super::{GraphManager, NodeLabel, make_node};
use crate::node::{NoiseGeneratorNode, SimpleWaveGeneratorNode, StingrayNode};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Generator, GeneratorInstance};
use state::GeneratorSelector;

/// Describes a generator from the viewpoint of the graph.
/// Contains a reference to the generator node.
#[derive(Clone)]
pub struct GeneratorInfo {
    pub selector: GeneratorSelector,
    node: NodeIndex,
}

impl GeneratorInfo {
    pub fn new(
        graph_manager: &mut GraphManager,
        generator: &GeneratorInstance,
        selector: GeneratorSelector,
    ) -> Self {
        let node = match &generator.it {
            Generator::SimpleWave(..) => make_node(SimpleWaveGeneratorNode::new(selector)),
            Generator::Noise(..) => make_node(NoiseGeneratorNode::new(selector)),
            Generator::Stingray(..) => make_node(StingrayNode::new(selector)),
        };

        let node = graph_manager.add_node(node, NodeLabel::Generator);

        Self { selector, node }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}
