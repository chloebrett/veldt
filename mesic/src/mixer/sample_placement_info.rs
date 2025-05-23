use super::make_node;
use crate::graph::Graph;
use crate::node::SampleNode;
use petgraph::stable_graph::NodeIndex;
use state::PlacementSelector;

/// Describes a sample placement from the viewpoint of the graph.
/// Contains a reference to the sample node.
#[derive(Clone)]
pub struct SamplePlacementInfo {
    node: NodeIndex,
}

impl SamplePlacementInfo {
    pub fn new(graph: &mut Graph, selector: PlacementSelector) -> Self {
        let node = graph.add_node(make_node(SampleNode::new(selector)));

        Self { node }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}
