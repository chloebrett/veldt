use super::{GraphManager, NodeLabel, make_node};
use crate::node::DrumTrackNode;
use petgraph::stable_graph::NodeIndex;
use state::PlacementSelector;

/// Describes a drum track from the viewpoint of the graph.
/// Contains a reference to the drum track node.
#[derive(Clone)]
pub struct DrumTrackInfo {
    node: NodeIndex,
}

impl DrumTrackInfo {
    pub fn new(graph_manager: &mut GraphManager, selector: PlacementSelector) -> Self {
        let node = graph_manager.add_node(make_node(DrumTrackNode::new(selector)), NodeLabel::Sample);

        Self { node }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}
