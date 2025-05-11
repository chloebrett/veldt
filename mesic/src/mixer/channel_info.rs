use std::collections::HashSet;

use super::{EdgeCounter, EdgeKey, EffectInfo, GeneratorInfo, make_node};
use crate::graph::Graph;
use crate::node::AmpNode;
use dasp_graph::node::Sum;
use petgraph::stable_graph::NodeIndex;
use shared::model::{EffectInstance, MatrixCell, Project};
use state::{EffectSelector, GeneratorSelector, MixerMatrixCellSelector, MixerSelector, move_elem};

/// Describes a mixer channel from the viewpoint of the graph.
/// Contains references to the generator and effect nodes linked to this channel.
pub struct ChannelInfo {
    // TODO: consider using a HashSet instead.
    generators: Vec<GeneratorInfo>,

    // Generators that have been muted and so should not have edges.
    muted_generators: HashSet<usize>,

    // Input sum node for this mixer channel.
    // Sums together the generators.
    pub input_node: NodeIndex,

    // Effect/mixer pairs for this channel.
    // Index = ordering within the channel.
    effects: Vec<EffectInfo>,

    // Output amp node for this mixer channel.
    pub output_node: NodeIndex,

    // Output routes for this mixer channel.
    // Indexes correspond to other mixer channels.
    // Values of None correspond to no route.
    // None values are necessarily the case for
    // (a) all outputs from the main channel, and
    // (b) each output from a channel to itself.
    // Nodes are AmpNodes, which control the volume sent from each channel to each other channel.
    pub output_routes: Vec<Option<NodeIndex>>,
}

impl ChannelInfo {
    pub fn new(graph: &mut Graph, project: &Project, channel_index: usize) -> Self {
        let mut muted_generators = HashSet::new();
        let generators: Vec<GeneratorInfo> = project
            .generators
            .iter()
            .filter(|generator| generator.meta.mixer_channel == channel_index)
            .enumerate()
            .map(|(generator_index, generator)| {
                if generator.meta.volume == 0.0 || generator.meta.mute {
                    muted_generators.insert(generator_index);
                }
                GeneratorInfo::new(graph, generator, GeneratorSelector(generator_index))
            })
            .collect();

        let input_node = graph.add_node(make_node(Sum));

        let effects: Vec<EffectInfo> = project.mixer.channels[channel_index]
            .effects
            .iter()
            .enumerate()
            .map(|(effect_index, effect)| {
                EffectInfo::new(graph, effect, &EffectSelector(channel_index, effect_index))
            })
            .collect();

        let output_node = graph.add_node(make_node(AmpNode::new_for_channel(MixerSelector(
            channel_index,
        ))));

        let mut partial = Self {
            generators,
            muted_generators,
            input_node,
            effects,
            output_node,
            output_routes: vec![],
        };
        partial.refresh_routes(graph, channel_index, project);
        partial
    }

    pub fn refresh_routes(&mut self, graph: &mut Graph, channel_index: usize, project: &Project) {
        // Delete existing route nodes, before adding new ones!
        // This is reasonably fine because the nodes are fairly small and stateless.
        // It still needs some allocations though, so we could be a bit pickier / more efficient if
        // we wanted to be. E.g. doing this on *every* matrix knob change isn't particularly
        // efficient. We should only do it when disconnecting/reconnecting completely, and even
        // then, only change the relevant node.
        // This would need a lot of unit testing to make sure it was correct.
        for route in self.output_routes.iter().flatten() {
            graph.remove_node(*route);
        }

        let row = channel_index;
        let matrix = &project.mixer.matrix;
        self.output_routes = (0..matrix.channels)
            .map(|col| {
                let default: MatrixCell = 0.0.into();
                let cell: &MatrixCell = matrix.get(row, col).unwrap_or(&default);
                let cell: f32 = (*cell).into();
                if cell != 0.0 {
                    let selector = MixerMatrixCellSelector(row, col);
                    let node = graph.add_node(make_node(AmpNode::new_for_route(selector)));
                    Some(node)
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn add_edges(&self, graph: &mut Graph, edge_counter: &mut EdgeCounter) {
        for (generator_index, generator) in self.generators.iter().enumerate() {
            if !self.muted_generators.contains(&generator_index) {
                // Do not add edges for muted generators.
                edge_counter.add_edge(
                    graph,
                    generator.node(),
                    self.input_node,
                    EdgeKey::GenToMixIn,
                );
            }
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
                first.wet_dry_node,
                EdgeKey::MixInToEffMix,
            );

            edge_counter.add_edge(
                graph,
                last.wet_dry_node,
                self.output_node,
                EdgeKey::EffMixToMixOut,
            );
        }

        // Link up the channel's outputs to its routes.
        // .flatten() ignores the None nodes.
        for route in self.output_routes.iter().flatten() {
            edge_counter.add_edge(graph, self.output_node, *route, EdgeKey::MixOutToRoute);
        }
    }

    pub fn contains_generator(&mut self, selector: GeneratorSelector) -> bool {
        self.generators
            .iter()
            .any(|generator| generator.selector == selector)
    }

    pub fn set_generator_muted(&mut self, generator_index: usize, mute: bool) {
        if mute {
            self.muted_generators.insert(generator_index);
        } else {
            self.muted_generators.remove(&generator_index);
        }
    }

    pub fn effects_count(&self) -> usize {
        self.effects.len()
    }

    pub fn move_effect(&mut self, from_index: usize, to_index: usize) {
        move_elem(&mut self.effects, from_index, to_index);
    }

    pub fn delete_effect(&mut self, graph: &mut Graph, index: usize) {
        let mut effect = self.effects.remove(index);
        effect.remove_from(graph);
    }

    pub fn add_effect(
        &mut self,
        graph: &mut Graph,
        effect: &EffectInstance,
        selector: &EffectSelector,
    ) {
        // EffectInfo::new handles adding nodes to the graph.
        self.effects.push(EffectInfo::new(graph, effect, selector));
    }

    /// Deletes a generator from the ChannelInfo's generator list, without deleting it from the graph.
    /// This allows for a two-step process whereby a generator is soft-deleted from one
    /// ChannelInfo then added to another, by reference, without actually recreating the generator.
    pub fn soft_delete_generator(&mut self, selector: GeneratorSelector) -> Option<GeneratorInfo> {
        for i in 0..self.generators.len() {
            if self.generators[i].selector == selector {
                // Note: swap_remove used because order of the generators doesn't matter,
                // but the performance gain of this is negligible.
                return Some(self.generators.swap_remove(i));
            }
        }
        None
    }

    /// Adds a generator to the ChannelInfo's generator list, without re-adding it to the graph.
    /// Designed to be used in tandem with soft_delete_generator for moving generator nodes
    /// between mixer channels.
    pub fn soft_add_generator(&mut self, generator: &GeneratorInfo) {
        self.generators.push(generator.clone());
    }

    pub fn route_to_inputs(
        &self,
        graph: &mut Graph,
        edge_counter: &mut EdgeCounter,
        inputs: &[NodeIndex],
    ) {
        for (input_channel_index, route_start) in self.output_routes.iter().enumerate() {
            if let Some(route_start) = route_start {
                let route_end = inputs[input_channel_index];
                edge_counter.add_edge(graph, *route_start, route_end, EdgeKey::RouteToMixIn);
            }
        }
    }
}
