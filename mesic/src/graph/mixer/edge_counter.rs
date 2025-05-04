use crate::graph::Graph;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

/// Counts edges in the graph by type.
/// Helpful for testing/debugging.
/// Note: consider extending this to also count nodes, if that would be helpful.
#[derive(Debug, Default)]
pub struct EdgeCounter {
    pub counts: HashMap<EdgeKey, usize>,
}

impl EdgeCounter {
    pub fn add_edge(&mut self, graph: &mut Graph, from: NodeIndex, to: NodeIndex, key: EdgeKey) {
        graph.add_edge(from, to, ());
        let current = self.counts.get(&key).unwrap_or(&0);
        self.counts.insert(key, current + 1);
        log::info!("Added edge: {:?}", key);
    }

    pub fn reset(&mut self) {
        self.counts.clear();
        log::info!("Reset edge counter.");
    }
}

/// The types of edges that can be added to the mixer graph.
/// Used for counting and debugging.
#[derive(Hash, Debug, Eq, PartialEq, Copy, Clone)]
pub enum EdgeKey {
    GenToMixIn,
    MixInToEff,
    MixInToEffMix,
    EffToEffMix,
    EffMixToNextEff,
    EffMixToNextEffMix,
    EffMixToMixOut,
    MixInToMixOut,
    MixOutToMainSum,
    MainSumToMainAmp,
    MainBufToMainSum,
}
