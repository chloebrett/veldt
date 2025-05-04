use super::{EdgeCounter, EdgeKey, make_node};
use crate::graph::{CompressorNode, DelayNode, EqNode, Graph, MixerNode, ModDelayNode};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance};
use state::EffectSelector;

#[expect(dead_code)] // Will need to read fields to manipulate later.
#[derive(Debug)]
pub struct EffectInfo {
    // Effect index within the project model.
    effect_index: usize,

    pub effect_node: NodeIndex,
    pub mixer_node: NodeIndex,
}

impl EffectInfo {
    pub fn new(
        graph: &mut Graph,
        edge_counter: &mut EdgeCounter,
        effect: &EffectInstance,
        effect_sel: &EffectSelector,
    ) -> Self {
        // TODO: just pass the selector down directly to the effect and mixer nodes.
        let EffectSelector(mixer_index, effect_index) = *effect_sel;

        let effect_node = match &effect.it {
            Effect::SimpleEq(config) => {
                make_node(EqNode::new(mixer_index, effect_index, config.clone()))
            }
            Effect::Delay(config) => {
                make_node(DelayNode::new(mixer_index, effect_index, config.clone()))
            }
            Effect::Compressor(config) => make_node(CompressorNode::new(config.clone())),
            Effect::ModDelay(config) => make_node(ModDelayNode::new(config.clone())),
        };
        let mixer_node = make_node(MixerNode::new(
            mixer_index,
            effect_index,
            effect.meta.clone(),
        ));

        let effect_node = graph.add_node(effect_node);
        let mixer_node = graph.add_node(mixer_node);
        edge_counter.add_edge(graph, effect_node, mixer_node, EdgeKey::EffToEffMix);

        Self {
            effect_index,
            effect_node,
            mixer_node,
        }
    }

    fn effect_node(&self) -> NodeIndex {
        self.effect_node
    }

    fn mixer_node(&self) -> NodeIndex {
        self.mixer_node
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
            self.mixer_node,
            next_effect.effect_node(),
            EdgeKey::EffMixToNextEff,
        );
        edge_counter.add_edge(
            graph,
            self.mixer_node,
            next_effect.mixer_node(),
            EdgeKey::EffMixToNextEffMix,
        );
    }
}
