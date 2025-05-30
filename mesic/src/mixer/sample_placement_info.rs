use super::{GraphManager, NodeLabel, make_node};
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
    pub fn new(graph_manager: &mut GraphManager, selector: PlacementSelector) -> Self {
        let node = graph_manager.add_node(make_node(SampleNode::new(selector)), NodeLabel::Sample);

        Self { node }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}
