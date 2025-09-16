use super::{GraphManager, NodeLabel, make_node};
use crate::node::DrumTrackPlacementNode;
use petgraph::stable_graph::NodeIndex;
use state::PlacementSelector;

/// Describes a drum track placement from the viewpoint of the graph.
/// Contains a reference to the drum track node.
#[derive(Clone)]
pub struct DrumTrackPlacementInfo {
    node: NodeIndex,
}

impl DrumTrackPlacementInfo {
    pub fn new(graph_manager: &mut GraphManager, selector: PlacementSelector) -> Self {
        let node = graph_manager.add_node(
            make_node(DrumTrackPlacementNode::new(selector)),
            NodeLabel::DrumTrack,
        );
        Self { node }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}
