use super::{EdgeLabel, GraphManager, NodeLabel, make_node};
use crate::node::{CompressorNode, DelayNode, EqNode, ModDelayNode, WetDryNode};
use petgraph::stable_graph::NodeIndex;
use shared::model::Effect;
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
    pub fn new(graph_manager: &mut GraphManager, effect: &Effect, sel: &EffectSelector) -> Self {
        let effect_node = match &effect {
            Effect::SimpleEq(_) => make_node(EqNode::new(*sel)),
            Effect::Delay(_) => make_node(DelayNode::new(*sel)),
            Effect::Compressor(_) => make_node(CompressorNode::new(*sel)),
            Effect::ModDelay(_) => make_node(ModDelayNode::new(*sel)),
        };

        let wet_dry_node = make_node(WetDryNode::new(*sel));

        let effect_node = graph_manager.add_node(effect_node, NodeLabel::Effect);
        let wet_dry_node = graph_manager.add_node(wet_dry_node, NodeLabel::WetDry);
        let EffectSelector(_, effect_index) = *sel;

        Self {
            effect_index,
            effect_node,
            wet_dry_node,
        }
    }

    /// Removes the effect and its wet/dry mixer from the graph.
    pub fn remove_from_graph(&mut self, graph_manager: &mut GraphManager) {
        graph_manager.remove_node(self.effect_node, NodeLabel::Effect);
        graph_manager.remove_node(self.wet_dry_node, NodeLabel::WetDry);
    }

    pub fn add_edges(&self, graph_manager: &mut GraphManager) {
        graph_manager.add_edge(
            self.effect_node,
            self.wet_dry_node,
            EdgeLabel::EffToEffWetDry,
        );
    }

    pub fn link_to(&self, next_effect: &EffectInfo, graph_manager: &mut GraphManager) {
        // TODO: confirm this results in the correct direction for wet/dry.
        graph_manager.add_edge(
            self.wet_dry_node,
            next_effect.effect_node,
            EdgeLabel::EffWetDryToNextEff,
        );
        graph_manager.add_edge(
            self.wet_dry_node,
            next_effect.wet_dry_node,
            EdgeLabel::EffWetDryToNextEffWetDry,
        );
    }
}
