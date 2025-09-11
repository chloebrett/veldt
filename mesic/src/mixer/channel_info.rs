use std::collections::HashSet;

use super::{
    EdgeLabel, EffectInfo, GeneratorInfo, GraphManager, NodeLabel, SamplePlacementInfo, DrumTrackInfo, make_node,
};
use crate::node::AmpNode;
use dasp_graph::node::Sum;
use petgraph::stable_graph::NodeIndex;
use shared::model::{
    Effect, EffectId, GeneratorId, MatrixCell, PlacementId, PlacementType, Project,
};
use state::{
    EffectSelector, GeneratorSelector, MixerMatrixCellSelector, MixerSelector, PlacementSelector,
    move_elem,
};
use std::collections::HashMap;

/// Describes a mixer channel from the viewpoint of the graph.
/// Contains references to the generator and effect nodes linked to this channel.
pub struct ChannelInfo {
    generators: HashMap<GeneratorId, GeneratorInfo>,

    // Generators that have been muted and so should not have edges.
    muted_generators: HashSet<GeneratorId>,

    samples: HashMap<PlacementId, SamplePlacementInfo>,
    drum_tracks: HashMap<PlacementId, DrumTrackInfo>,
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
    pub fn new(graph_manager: &mut GraphManager, project: &Project, channel_index: usize) -> Self {
        let mut muted_generators = HashSet::new();
        let generators: HashMap<GeneratorId, GeneratorInfo> = project
            .generators
            .iter()
            .filter(|(_, generator)| generator.meta.mixer_channel == channel_index)
            .map(|(generator_id, generator)| {
                if generator.meta.volume == 0.0 || generator.meta.mute {
                    muted_generators.insert(*generator_id);
                }
                (
                    *generator_id,
                    GeneratorInfo::new(graph_manager, generator, GeneratorSelector(*generator_id)),
                )
            })
            .collect();

        // TODO: let each sample placement choose which mixer channel it is on, instead of putting
        // all sample placements on channel 0.
        let mut samples: HashMap<PlacementId, SamplePlacementInfo> = HashMap::new();
        if channel_index == 0 {
            let _ = project
                .placements
                .iter()
                .filter(|(_, placement)| matches!(&placement.kind, PlacementType::Sample(..)))
                .map(|(id, _)| {
                    samples.insert(
                        *id,
                        SamplePlacementInfo::new(graph_manager, PlacementSelector(*id)),
                    )
                });
        };

        let mut drum_tracks: HashMap<PlacementId, DrumTrackInfo> = HashMap::new();
        if channel_index == 0 {
            let _ = project
                .placements
                .iter()
                .filter(|(_, placement)| matches!(&placement.kind, PlacementType::DrumTrack(..)))
                .map(|(id, _)| {
                    drum_tracks.insert(
                        *id,
                        DrumTrackInfo::new(graph_manager, PlacementSelector(*id)),
                    )
                });
        };

        let input_node = graph_manager.add_node(make_node(Sum), NodeLabel::Sum);

        let effects: Vec<EffectInfo> = project.mixer.channels[channel_index]
            .effect_ids
            .iter()
            .map(|effect_id| {
                let effect = &project.effects[effect_id].it;
                EffectInfo::new(graph_manager, effect, &EffectSelector(*effect_id))
            })
            .collect();

        let output_node = graph_manager.add_node(
            make_node(AmpNode::new_for_channel(MixerSelector(channel_index))),
            NodeLabel::Amp,
        );

        let mut partial = Self {
            generators,
            muted_generators,
            samples,
            drum_tracks,
            input_node,
            effects,
            output_node,
            output_routes: vec![],
        };
        partial.refresh_routes(graph_manager, channel_index, project);
        partial
    }

    pub fn refresh_routes(
        &mut self,
        graph_manager: &mut GraphManager,
        channel_index: usize,
        project: &Project,
    ) {
        // Delete existing route nodes, before adding new ones!
        // This is reasonably fine because the nodes are fairly small and stateless.
        // It still needs some allocations though, so we could be a bit pickier / more efficient if
        // we wanted to be. E.g. doing this on *every* matrix knob change isn't particularly
        // efficient. We should only do it when disconnecting/reconnecting completely, and even
        // then, only change the relevant node.
        // This would need a lot of unit testing to make sure it was correct.
        for route in self.output_routes.iter().flatten() {
            graph_manager.remove_node(*route, NodeLabel::Amp);
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
                    let node = graph_manager
                        .add_node(make_node(AmpNode::new_for_route(selector)), NodeLabel::Amp);
                    Some(node)
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn add_edges(&self, graph_manager: &mut GraphManager) {
        for (generator_id, generator) in self.generators.iter() {
            if !self.muted_generators.contains(generator_id) {
                // Do not add edges for muted generators.
                graph_manager.add_edge(generator.node(), self.input_node, EdgeLabel::GenToMixIn);
            }
        }

        for sample in self.samples.values() {
            graph_manager.add_edge(sample.node(), self.input_node, EdgeLabel::SampleToMixIn);
        }

        let effects = &self.effects;
        for effect in effects {
            effect.add_edges(graph_manager);
        }

        // TODO: get .zip() working.
        if !effects.is_empty() {
            for i in 0..effects.len() - 1 {
                let effect = &effects[i];
                let next_effect = &effects[i + 1];
                effect.link_to(next_effect, graph_manager);
            }
        }

        // Link up the input -> effects -> output.
        // If there are no effects, link directly from input -> output.
        if effects.is_empty() {
            graph_manager.add_edge(self.input_node, self.output_node, EdgeLabel::MixInToMixOut);
        } else {
            let first = effects.first().unwrap();
            let last = effects.last().unwrap();

            // TODO: check the wet/dry direction here.
            graph_manager.add_edge(self.input_node, first.effect_node, EdgeLabel::MixInToEff);
            graph_manager.add_edge(
                self.input_node,
                first.wet_dry_node,
                EdgeLabel::MixInToEffWetDry,
            );

            graph_manager.add_edge(
                last.wet_dry_node,
                self.output_node,
                EdgeLabel::EffWetDryToMixOut,
            );
        }

        // Link up the channel's outputs to its routes.
        // .flatten() ignores the None nodes.
        for route in self.output_routes.iter().flatten() {
            graph_manager.add_edge(self.output_node, *route, EdgeLabel::MixOutToRoute);
        }
    }

    pub fn contains_generator(&mut self, selector: GeneratorSelector) -> bool {
        let GeneratorSelector(generator_id) = selector;
        self.generators.contains_key(&generator_id)
    }

    pub fn set_generator_muted(&mut self, generator_id: GeneratorId, mute: bool) {
        if mute {
            self.muted_generators.insert(generator_id);
        } else {
            self.muted_generators.remove(&generator_id);
        }
    }

    pub fn move_effect(&mut self, from_index: usize, to_index: usize) {
        move_elem(&mut self.effects, from_index, to_index);
    }

    pub fn delete_effect(&mut self, graph_manager: &mut GraphManager, effect_id: EffectId) {
        let Some(index) = self
            .effects
            .iter()
            .position(|it| it.effect_id() == effect_id)
        else {
            return;
        };
        let mut effect = self.effects.remove(index);
        effect.remove_from_graph(graph_manager);
    }

    pub fn add_effect(
        &mut self,
        graph_manager: &mut GraphManager,
        effect: &Effect,
        selector: &EffectSelector,
        at_index: usize,
    ) {
        // EffectInfo::new handles adding nodes to the graph.
        self.effects
            .insert(at_index, EffectInfo::new(graph_manager, effect, selector));
    }

    /// Deletes a generator from the ChannelInfo's generator list, without deleting it from the graph.
    /// This allows for a two-step process whereby a generator is soft-deleted from one
    /// ChannelInfo then added to another, by reference, without actually recreating the generator.
    pub fn soft_delete_generator(&mut self, selector: GeneratorSelector) -> Option<GeneratorInfo> {
        let GeneratorSelector(generator_id) = selector;
        self.generators.remove(&generator_id)
    }

    /// Adds a generator to the ChannelInfo's generator list, without re-adding it to the graph.
    /// Designed to be used in tandem with soft_delete_generator for moving generator nodes
    /// between mixer channels.
    pub fn soft_add_generator(&mut self, generator: &GeneratorInfo) {
        let GeneratorSelector(generator_id) = generator.selector;
        self.generators.insert(generator_id, generator.clone());
    }

    pub fn route_to_inputs(&self, graph_manager: &mut GraphManager, inputs: &[NodeIndex]) {
        for (input_channel_index, route_start) in self.output_routes.iter().enumerate() {
            if let Some(route_start) = route_start {
                let route_end = inputs[input_channel_index];
                graph_manager.add_edge(*route_start, route_end, EdgeLabel::RouteToMixIn);
            }
        }
    }

    pub fn soft_add_placement_sample(&mut self, graph_manager: &mut GraphManager, id: PlacementId) {
        self.samples.insert(
            id,
            SamplePlacementInfo::new(graph_manager, PlacementSelector(id)),
        );
    }
}
