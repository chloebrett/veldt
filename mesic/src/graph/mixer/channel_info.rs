use super::{EdgeCounter, EdgeKey, EffectInfo, GeneratorInfo, make_node};
use crate::graph::{AmpNode, Graph};
use dasp_graph::node::Sum;
use petgraph::stable_graph::NodeIndex;
use shared::model::Project;
use state::EffectSelector;

/// Describes a mixer channel from the viewpoint of the graph.
/// Contains references to the generator and effect nodes linked to this channel.
pub struct ChannelInfo {
    // TODO: consider using a HashSet instead.
    generators: Vec<GeneratorInfo>,

    // Input sum node for this mixer channel.
    // Sums together the generators.
    input_node: NodeIndex,

    // Effect/mixer pairs for this channel.
    // Index = ordering within the channel.
    effects: Vec<EffectInfo>,

    // Output amp node for this mixer channel.
    pub output_node: NodeIndex,
}

impl ChannelInfo {
    pub fn new(graph: &mut Graph, project: &Project, channel_index: usize) -> Self {
        let generators: Vec<GeneratorInfo> = project
            .generators
            .iter()
            .filter(|generator| generator.meta.mixer_channel == channel_index)
            .enumerate()
            .map(|(generator_index, generator)| {
                GeneratorInfo::new(graph, project, generator, generator_index)
            })
            .collect();

        let input_node = graph.add_node(make_node(Sum));

        let effects: Vec<EffectInfo> = project.mixer[channel_index]
            .effects
            .iter()
            .enumerate()
            .map(|(effect_index, effect)| {
                EffectInfo::new(graph, effect, &EffectSelector(channel_index, effect_index))
            })
            .collect();

        // TODO: wire up the amp node to read the correct volume.
        let output_node = graph.add_node(make_node(AmpNode::default()));

        Self {
            generators,
            input_node,
            effects,
            output_node,
        }
    }

    pub fn add_edges(&self, graph: &mut Graph, edge_counter: &mut EdgeCounter) {
        for generator in &self.generators {
            edge_counter.add_edge(
                graph,
                generator.node(),
                self.input_node,
                EdgeKey::GenToMixIn,
            );
        }

        let effects = &self.effects;
        for effect in effects {
            effect.add_edges(graph, edge_counter);
        }

        // TODO: get .zip() working.
        if !effects.is_empty() {
            for i in 0..effects.len() - 1 {
                let effect = &effects[i];
                let next_effect = &effects[i + 1];
                effect.link_to(next_effect, graph, edge_counter);
            }
        }

        // Link up the input -> effects -> output.
        // If there are no effects, link directly from input -> output.
        if effects.is_empty() {
            edge_counter.add_edge(
                graph,
                self.input_node,
                self.output_node,
                EdgeKey::MixInToMixOut,
            );
        } else {
            let first = effects.first().unwrap();
            let last = effects.last().unwrap();

            // TODO: check the wet/dry direction here.
            edge_counter.add_edge(
                graph,
                self.input_node,
                first.effect_node,
                EdgeKey::MixInToEff,
            );
            edge_counter.add_edge(
                graph,
                self.input_node,
                first.mixer_node,
                EdgeKey::MixInToEffMix,
            );

            edge_counter.add_edge(
                graph,
                last.mixer_node,
                self.output_node,
                EdgeKey::EffMixToMixOut,
            );
        }
    }
}
