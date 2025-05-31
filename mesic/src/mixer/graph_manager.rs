use super::ProcessContext;
use crate::graph::Graph;
use dasp_graph::{BoxedNodeSend, NodeData};
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

/// Holds a graph and tracks node and edges with extra metadata.
/// Helpful for testing/debugging.
pub struct GraphManager {
    pub graph: Graph,
    pub node_counts: HashMap<NodeLabel, usize>,
    pub edge_counts: HashMap<EdgeLabel, usize>,
}

impl GraphManager {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            node_counts: HashMap::new(),
            edge_counts: HashMap::new(),
        }
    }

    pub fn add_node(
        &mut self,
        node: NodeData<BoxedNodeSend<ProcessContext>>,
        key: NodeLabel,
    ) -> NodeIndex {
        let added = self.graph.add_node(node);
        let current = self.node_counts.get(&key).unwrap_or(&0);
        self.node_counts.insert(key, current + 1);
        log::info!("Added node: {:?}", key);
        added
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, key: EdgeLabel) {
        self.graph.add_edge(from, to, ());
        let current = self.edge_counts.get(&key).unwrap_or(&0);
        self.edge_counts.insert(key, current + 1);
        log::info!("Added edge: {:?}", key);
    }

    pub fn remove_node(&mut self, node: NodeIndex, key: NodeLabel) {
        self.graph.remove_node(node);

        // TODO: abstract this better.
        let current = self.node_counts.get(&key).unwrap_or(&0);
        self.node_counts.insert(key, current - 1);
        if *self.node_counts.get(&key).unwrap() == 0 {
            self.node_counts.remove(&key);
        }
    }

    pub fn clear_edges(&mut self) {
        self.graph.clear_edges();
        self.edge_counts.clear();
        log::info!("Reset edges on graph manager.");
    }
}

/// The types of nodes that can be added to the mixer graph.
/// Used for counting and debugging.
#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone)]
pub enum NodeLabel {
    Generator,
    Sample,
    Effect,
    WetDry,
    Sum,
    Amp,
    Buffer,
}

/// The types of edges that can be added to the mixer graph.
/// Used for counting and debugging.
#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone)]
pub enum EdgeLabel {
    GenToMixIn,
    SampleToMixIn,
    MixInToEff,
    MixInToEffWetDry,
    EffToEffWetDry,
    EffWetDryToNextEff,
    EffWetDryToNextEffWetDry,
    EffWetDryToMixOut,
    MixInToMixOut,
    MixOutToRoute,
    RouteToMixIn,
    MixOutToMainSum,
    MainSumToMainAmp,
    MainBufToMainSum,
}
