use super::make_node;
use crate::graph::Graph;
use crate::node::{SimpleWaveGeneratorNode, SubSynthNode};
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
        graph: &mut Graph,
        generator: &GeneratorInstance,
        selector: GeneratorSelector,
    ) -> Self {
        let node = match &generator.it {
            Generator::SimpleWave(..) => make_node(SimpleWaveGeneratorNode::new(selector)),
            Generator::SubSynth(..) => make_node(SubSynthNode::new(selector)),
            _ => {
                // TODO: support adding other types of generators to the graph.
                panic!("Not yet implemented.")
            }
        };

        let node = graph.add_node(node);

        Self { selector, node }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}
