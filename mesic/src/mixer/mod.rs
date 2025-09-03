use crate::graph::{ProcessContext, Processor, make_graph};
use crate::node::{AmpNode, BufferNode};
use crossbeam_channel::Sender;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::Project;
use state::{
    Action, EffectSelector, FloatField, GeneratorSelector, IndexField, MoveField, Selector,
    StoreData, TypeField,
};

mod channel_info;
mod effect_info;
mod generator_info;
mod graph_manager;
mod sample_placement_info;

use channel_info::*;
use effect_info::*;
use generator_info::*;
use graph_manager::*;
use sample_placement_info::*;

pub use graph_manager::{GraphDebugInfo, NodeLabel};

/// The mixer is responsible for creating, storing and manipulating mixer channels,
/// and the effects and generators they contain.
///
/// The mixer arrangement looks like this:
///
/// ```text
/// g  g // generators linked to mixer channel 1
/// |  |
/// |---
/// v
/// s ---- // mixer channel 1 in
/// |    |
/// |    |         ----------|
/// v    v         |         v
/// e -> m -> e -> m -> e -> m // mixer effects chain (with wet/dry mixer nodes)
///      |         ^         |
///      ----------|         |
///                          |
/// |-------------------------
/// v
/// a // mixer channel 1 out (amp)
/// |
/// | // note: more nodes will be added here in future for channel routing
/// v
/// s // main sum (TODO: remove this as it's redundant now that only the main channel uses it).
/// |
/// v
/// a // main out
/// ```
///
/// Note: the diagram above does not account for mixers routing to each other!
///
/// ------------
/// a = amp node
/// e = effect node
/// m = mixer node
/// g = generator node
/// s = sum node
pub struct Mixer {
    pub graph_manager: GraphManager,

    channels: Vec<ChannelInfo>,

    // Main buffer node, for playing arbitrary audio buffers (e.g. sample preview).
    main_buffer: NodeIndex,

    // Main sum node.
    // Adds up the main_buffer with the main channel output.
    // Directs its output to the main_amp node.
    main_sum: NodeIndex,

    // Main output amp node.
    main_amp: NodeIndex,
}

impl Mixer {
    pub fn new(project: &Project, debug_tx: Option<Sender<GraphDebugInfo>>) -> Self {
        let mut graph_manager = GraphManager::new(make_graph());
        if let Some(debug_tx) = debug_tx {
            graph_manager.set_debug_tx(debug_tx);
        }

        let channels: Vec<ChannelInfo> = (0..project.mixer.channels.len())
            .map(|channel_index| ChannelInfo::new(&mut graph_manager, project, channel_index))
            .collect();

        let main_buffer =
            graph_manager.add_node(make_node(BufferNode::default()), NodeLabel::Buffer);
        let main_sum = graph_manager.add_node(make_node(Sum), NodeLabel::Sum);
        let main_amp = graph_manager.add_node(make_node(AmpNode::new_main()), NodeLabel::Amp);

        Self {
            graph_manager,
            channels,
            main_buffer,
            main_sum,
            main_amp,
        }
        .with_refreshed_edges()
    }

    fn with_refreshed_edges(self) -> Self {
        let mut mixer = self;
        mixer.refresh_edges();
        mixer
    }

    /// Clears all the edges in the graph and re-evaluates them based on the arrangement of nodes.
    /// The edges are a pure function of the current mixer state (determined by the arrangement of
    /// the ChannelInfos and the structs contained within them).
    pub fn refresh_edges(&mut self) {
        self.graph_manager.clear_edges();

        self.graph_manager
            .add_edge(self.main_buffer, self.main_sum, EdgeLabel::MainBufToMainSum);

        let inputs: Vec<_> = self
            .channels
            .iter()
            .map(|channel| channel.input_node)
            .collect();

        for (channel_index, channel) in self.channels.iter().enumerate() {
            channel.add_edges(&mut self.graph_manager);

            // Only add the main channel to the main output sum node.
            // Other channels need to be routed via main.
            if channel_index == 0 {
                self.graph_manager.add_edge(
                    channel.output_node,
                    self.main_sum,
                    EdgeLabel::MixOutToMainSum,
                );
            } else {
                channel.route_to_inputs(&mut self.graph_manager, &inputs);
            }
        }

        self.graph_manager
            .add_edge(self.main_sum, self.main_amp, EdgeLabel::MainSumToMainAmp);
    }

    pub fn get_debug_tx(&self) -> Option<Sender<GraphDebugInfo>> {
        self.graph_manager.get_debug_tx()
    }

    pub fn set_debug_tx(&mut self, debug_tx: Sender<GraphDebugInfo>) {
        self.graph_manager.set_debug_tx(debug_tx);
    }

    /// Applies the given action, updating the underlying graph accordingly.
    /// If the graph changes, edges are refreshed.
    /// TODO: consider processing multiple actions at once, and only refreshing the edges a single
    /// time.
    pub fn update(&mut self, selector: &Selector, action: &Action, store: &StoreData) {
        let did_change = match selector {
            Selector::Mixer(mixer_index) => match action {
                Action::MoveChild(MoveField {
                    from_field,
                    to_field,
                }) => {
                    let (IndexField::EffectId(from), IndexField::EffectId(to)) =
                        (from_field, to_field)
                    else {
                        panic!("Action should have only received EffectId IndexFields.")
                    };
                    self.channels[*mixer_index].move_effect(*from, *to);
                    true
                }
                Action::DeleteChildById(TypeField::EffectId(effect_id)) => {
                    self.channels[*mixer_index].delete_effect(&mut self.graph_manager, *effect_id);
                    true
                }
                Action::AddChildAtIndex(
                    TypeField::EffectId(effect_id),
                    IndexField::EffectId(index),
                ) => {
                    log::info!("Add child at index");
                    let effect = &store.project.effects[effect_id].it;
                    self.channels[*mixer_index].add_effect(
                        &mut self.graph_manager,
                        effect,
                        &EffectSelector(*effect_id),
                        *index,
                    );
                    true
                }
                _ => false,
            },
            Selector::Generator(generator_id) => {
                let selector = GeneratorSelector(*generator_id);
                match action {
                    Action::SetIndex(IndexField::Mixer(mixer_channel)) => {
                        // Find the channel containing this generator, then move it to the correct
                        // channel.
                        for channel in self.channels.iter_mut() {
                            if let Some(generator) = channel.soft_delete_generator(selector) {
                                self.channels[*mixer_channel].soft_add_generator(&generator);
                                // TODO: Update mute information when generator is moved.
                                break;
                            }
                        }
                        true
                    }
                    Action::SetFloat(FloatField::Volume, 0.0)
                    | Action::SetChild(TypeField::Mute(true)) => {
                        // Find the channel containing this generator, then mute it.
                        for channel in self.channels.iter_mut() {
                            if channel.contains_generator(selector) {
                                channel.set_generator_muted(*generator_id, true);
                                break;
                            }
                        }
                        true
                    }
                    Action::SetFloat(FloatField::Volume, _)
                    | Action::SetChild(TypeField::Mute(false)) => {
                        // Find the channel containing this generator, then unmute it.
                        for channel in self.channels.iter_mut() {
                            if channel.contains_generator(selector) {
                                channel.set_generator_muted(*generator_id, false);
                                break;
                            }
                        }
                        true
                    }
                    _ => false,
                }
            }
            Selector::Placement(..) => {
                // TODO: handle adding, deleting and updating nodes when sample placements change.
                false
            }
            Selector::MixerMatrixCell(..) => match action {
                // If the matrix changes, reset the routes for each node, then refresh the edges.
                // NOTE: in future, consider what happens if the size of the matrix changes too.
                Action::SetFloat(FloatField::ModFactor, _) => {
                    for (channel_index, channel) in self.channels.iter_mut().enumerate() {
                        channel.refresh_routes(
                            &mut self.graph_manager,
                            channel_index,
                            &store.project,
                        );
                    }
                    true
                }
                _ => false,
            },
            Selector::Root => match action {
                Action::AddChild(TypeField::MixerChannel(..)) => {
                    let new_channel = ChannelInfo::new(
                        &mut self.graph_manager,
                        &store.project,
                        store.project.mixer.channels.len() - 1,
                    );
                    self.channels.push(new_channel);
                    true
                }
                Action::AddChild(TypeField::Generator(gen_instance)) => {
                    let mixer_index = gen_instance.meta.mixer_channel;
                    self.channels[mixer_index]
                        .add_new_generator(&mut self.graph_manager, gen_instance);
                    true
                }
                _ => false,
            },
            // TODO: handle adding and deleting generators (not just changing their mixer channel).
            _ => false,
        };

        if did_change {
            self.refresh_edges();
        }
    }

    /// Returns a vec of buffers corresponding to the main amp node at index 0 and all other mixer
    /// channels, which are filled after a processing run.
    /// Non-main channel outputs are used for visualising their level on the UI.
    pub fn output_buffers(&self) -> Vec<&Vec<Buffer>> {
        let mut channels: Vec<_> = self
            .channels
            .iter()
            .map(|channel| channel.output_node)
            .collect();
        // Set channel 0 to main amp.
        // TODO: Consider if the pre-amp main channel (channel 0) should be added as well as the main amp.
        channels[0] = self.main_amp;
        let buffers = channels
            .into_iter()
            .map(|node| &self.graph_manager.graph.node_weight(node).unwrap().buffers)
            .collect();
        buffers
    }

    /// Processes the graph.
    /// Exposed as a method so that we don't ever have to expose a mutable version of Graph.
    /// Therefore, the mixer is the only object allowed to mutate the Graph.
    pub fn process(&mut self, processor: &mut Processor, payload: &ProcessContext) {
        processor.process(&mut self.graph_manager.graph, payload, self.main_amp);
    }
}

fn make_node(
    node: impl Node<ProcessContext> + 'static + Send,
) -> NodeData<BoxedNodeSend<ProcessContext>> {
    NodeData::new2(BoxedNodeSend::new(node))
}

#[cfg(test)]
mod tests {
    use super::*;

    use shared::model::{
        DelayConfig, Effect, EffectId, EffectInstance, EffectMeta, Generator, GeneratorId,
        GeneratorInstance, GeneratorMeta, MixerChannel, SimpleWaveConfig,
    };
    use std::collections::HashMap;

    #[test]
    fn one_generator_one_effect() {
        let mut project = Project::default();
        project.generators.insert(GeneratorId(0), some_generator());
        project.effects.insert(EffectId(0), some_effect());
        project.mixer.channels.push(MixerChannel {
            volume: 1.0,
            mute: false,
            effect_ids: vec![EffectId(0)],
        });

        let mixer = Mixer::new(&project, None);

        let mut node_counts = HashMap::new();
        node_counts.insert(NodeLabel::Generator, 1);
        node_counts.insert(NodeLabel::Effect, 1);
        node_counts.insert(NodeLabel::WetDry, 1);
        node_counts.insert(NodeLabel::Buffer, 1);
        // Main output + channel output
        node_counts.insert(NodeLabel::Amp, 2);
        // Main sum + channel input
        node_counts.insert(NodeLabel::Sum, 2);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeLabel::GenToMixIn, 1);
        edge_counts.insert(EdgeLabel::MixInToEff, 1);
        edge_counts.insert(EdgeLabel::MixInToEffWetDry, 1);
        edge_counts.insert(EdgeLabel::EffToEffWetDry, 1);
        edge_counts.insert(EdgeLabel::EffWetDryToMixOut, 1);
        edge_counts.insert(EdgeLabel::MixOutToMainSum, 1);
        edge_counts.insert(EdgeLabel::MainBufToMainSum, 1);
        edge_counts.insert(EdgeLabel::MainSumToMainAmp, 1);

        for (key, count) in mixer.graph_manager.node_counts.iter() {
            assert_eq!(
                Some(count),
                node_counts.get(key),
                "Key: {:?}, had: {}, expected: {:?}",
                key,
                count,
                node_counts.get(key)
            );
        }

        // TODO: abstract out these HashMap assertions.
        for (key, count) in mixer.graph_manager.edge_counts.iter() {
            assert_eq!(
                Some(count),
                edge_counts.get(key),
                "Key: {:?}, had: {}, expected: {:?}",
                key,
                count,
                edge_counts.get(key)
            );
        }
        assert_eq!(mixer.channels.len(), 1);
    }

    #[test]
    fn multiple_generators_effects_channels() {
        let mut project = Project::default();

        // Two generators on channel 0,
        // One generator on channel 1.
        project
            .generators
            .extend((0..=2).map(|i| (GeneratorId(i), some_generator())));
        project
            .generators
            .get_mut(&GeneratorId(2))
            .unwrap()
            .meta
            .mixer_channel = 1;

        // One effect on channel 0,
        // Two effects on channel 1.
        project
            .effects
            .extend((0..=2).map(|i| (EffectId(i), some_effect())));
        project.mixer.channels.extend([
            MixerChannel {
                volume: 1.0,
                mute: false,
                effect_ids: vec![EffectId(0)],
            },
            MixerChannel {
                volume: 1.0,
                mute: false,
                effect_ids: vec![EffectId(1), EffectId(2)],
            },
        ]);

        let mixer = Mixer::new(&project, None);

        let mut node_counts = HashMap::new();
        node_counts.insert(NodeLabel::Generator, 3);
        node_counts.insert(NodeLabel::Buffer, 1);
        node_counts.insert(NodeLabel::Effect, 3);
        node_counts.insert(NodeLabel::WetDry, 3);
        // Main sum + one sum for each channel input
        node_counts.insert(NodeLabel::Sum, 3);
        // Main amp + one amp for each channel output
        node_counts.insert(NodeLabel::Amp, 3);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeLabel::GenToMixIn, 3);
        edge_counts.insert(EdgeLabel::MixInToEff, 2);
        edge_counts.insert(EdgeLabel::MixInToEffWetDry, 2);
        edge_counts.insert(EdgeLabel::EffToEffWetDry, 3);
        edge_counts.insert(EdgeLabel::EffWetDryToNextEff, 1);
        edge_counts.insert(EdgeLabel::EffWetDryToNextEffWetDry, 1);
        edge_counts.insert(EdgeLabel::EffWetDryToMixOut, 2);
        edge_counts.insert(EdgeLabel::MixOutToMainSum, 1);
        edge_counts.insert(EdgeLabel::MainBufToMainSum, 1);
        edge_counts.insert(EdgeLabel::MainSumToMainAmp, 1);

        for (key, count) in mixer.graph_manager.node_counts.iter() {
            assert_eq!(
                Some(count),
                node_counts.get(key),
                "{:?} {} {:?}",
                key,
                count,
                node_counts.get(key)
            );
        }
        for (key, count) in mixer.graph_manager.edge_counts.iter() {
            assert_eq!(
                Some(count),
                edge_counts.get(key),
                "{:?} {} {:?}",
                key,
                count,
                edge_counts.get(key)
            );
        }
        assert_eq!(mixer.channels.len(), 2);
    }

    // TODO: create defaults for each model object, to use in tests.
    fn some_effect() -> EffectInstance {
        let config = DelayConfig {
            delay_ms: 10.0,
            feedback: 0.5,
        };

        let meta = EffectMeta {
            wet: 1.0,
            mute: false,
        };

        EffectInstance {
            it: Effect::Delay(config),
            meta,
        }
    }

    // TODO: create defaults for each model object, to use in tests.
    fn some_generator() -> GeneratorInstance {
        GeneratorInstance {
            it: Generator::SimpleWave(SimpleWaveConfig::default()),
            meta: GeneratorMeta::default(),
        }
    }

    // TODO: write tests for updating the store with actions.
    // TODO: write tests for routing between mixer channels.
}
