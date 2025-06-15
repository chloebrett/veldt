use super::ProcessContext;
use crate::graph::Graph;
use crossbeam_channel::Sender;
use dasp_graph::{BoxedNodeSend, NodeData};
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GraphDebugInfo {
    // usize = node index
    pub node_labels: HashMap<usize, NodeLabel>,
    pub edges: Vec<(usize, usize)>,
}

/// Holds a graph and tracks node and edges with extra metadata.
/// Helpful for testing/debugging.
pub struct GraphManager {
    pub graph: Graph,
    pub node_counts: HashMap<NodeLabel, usize>,
    pub edge_counts: HashMap<EdgeLabel, usize>,

    // Used for rendering a graph for debugging.
    pub node_labels: HashMap<NodeIndex, NodeLabel>,
    debug_tx: Option<Sender<GraphDebugInfo>>,
}

impl GraphManager {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            node_counts: HashMap::new(),
            edge_counts: HashMap::new(),
            node_labels: HashMap::new(),
            debug_tx: None,
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
        self.node_labels.insert(added, key);
        log::info!("Added node: {:?}", key);
        self.send_debug();
        added
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, key: EdgeLabel) {
        self.graph.add_edge(from, to, ());
        let current = self.edge_counts.get(&key).unwrap_or(&0);
        self.edge_counts.insert(key, current + 1);
        log::info!("Added edge: {:?}", key);
        self.send_debug();
    }

    pub fn remove_node(&mut self, node: NodeIndex, key: NodeLabel) {
        self.graph.remove_node(node);

        // TODO: abstract this better.
        let current = self.node_counts.get(&key).unwrap_or(&0);
        self.node_counts.insert(key, current - 1);
        if *self.node_counts.get(&key).unwrap() == 0 {
            self.node_counts.remove(&key);
        }
        self.node_labels.remove(&node);
        self.send_debug();
    }

    pub fn clear_edges(&mut self) {
        self.graph.clear_edges();
        self.edge_counts.clear();
        log::info!("Reset edges on graph manager.");
        self.send_debug();
    }

    pub fn get_debug_tx(&self) -> Option<Sender<GraphDebugInfo>> {
        self.debug_tx.clone()
    }

    pub fn set_debug_tx(&mut self, debug_tx: Sender<GraphDebugInfo>) {
        self.debug_tx = Some(debug_tx);
        self.send_debug();
    }

    fn send_debug(&self) {
        let Some(tx) = &self.debug_tx else {
            log::info!("Send debug - missing debug tx!");
            return;
        };
        log::info!("Send debug - has debug tx!");

        let edges: Vec<(NodeIndex, NodeIndex)> = self
            .graph
            .edge_indices()
            .filter_map(|edge| self.graph.edge_endpoints(edge))
            .collect();
        tx.send(GraphDebugInfo {
            node_labels: self
                .node_labels
                .iter()
                .map(|(key, value)| (key.index(), *value))
                .collect(),
            edges: edges
                .iter()
                .map(|(first, second)| (first.index(), second.index()))
                .collect(),
        })
        .unwrap();
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
