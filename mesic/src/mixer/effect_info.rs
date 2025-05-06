use super::{EdgeCounter, EdgeKey, make_node};
use crate::graph::Graph;
use crate::node::{CompressorNode, DelayNode, EqNode, ModDelayNode, WetDryNode};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance};
use state::EffectSelector;

/// Describes an effect + wet/dry mixer from the viewpoint of the graph.
/// Contains references to the effect node and the wet/dry mixer node.
#[expect(dead_code)] // Will need to read fields to manipulate later.
#[derive(Debug, Clone)]
pub struct EffectInfo {
    // Effect index within the project model.
    effect_index: usize,

    pub effect_node: NodeIndex,
    pub wet_dry_node: NodeIndex,
}

impl EffectInfo {
    pub fn new(graph: &mut Graph, effect: &EffectInstance, sel: &EffectSelector) -> Self {
        let effect_node = match &effect.it {
            Effect::SimpleEq(config) => make_node(EqNode::new(*sel, config.clone())),
            Effect::Delay(config) => make_node(DelayNode::new(*sel, config.clone())),
            Effect::Compressor(config) => make_node(CompressorNode::new(config.clone())),
            Effect::ModDelay(config) => make_node(ModDelayNode::new(config.clone())),
        };

        let wet_dry_node = make_node(WetDryNode::new(*sel, effect.meta.clone()));

        let effect_node = graph.add_node(effect_node);
        let wet_dry_node = graph.add_node(wet_dry_node);
        let EffectSelector(_, effect_index) = *sel;

        Self {
            effect_index,
            effect_node,
            wet_dry_node,
        }
    }

    /// Removes the effect and its wet/dry mixer from the graph.
    pub fn remove_from(&mut self, graph: &mut Graph) {
        graph.remove_node(self.effect_node);
        graph.remove_node(self.wet_dry_node);
    }

    pub fn add_edges(&self, graph: &mut Graph, edge_counter: &mut EdgeCounter) {
        edge_counter.add_edge(
            graph,
            self.effect_node,
            self.wet_dry_node,
            EdgeKey::EffToEffMix,
        );
    }

    pub fn link_to(
        &self,
        next_effect: &EffectInfo,
        graph: &mut Graph,
        edge_counter: &mut EdgeCounter,
    ) {
        // TODO: confirm this results in the correct direction for wet/dry.
        edge_counter.add_edge(
            graph,
            self.wet_dry_node,
            next_effect.effect_node,
            EdgeKey::EffMixToNextEff,
        );
        edge_counter.add_edge(
            graph,
            self.wet_dry_node,
            next_effect.wet_dry_node,
            EdgeKey::EffMixToNextEffMix,
        );
    }
}
