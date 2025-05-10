use crate::graph::{Graph, ProcessContext, Processor, make_graph};
use crate::node::{AmpNode, BufferNode};
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::Project;
use state::{
    Action, EffectSelector, FloatField, GeneratorSelector, IndexField, MoveField, Selector,
    StoreData, TypeField,
};

mod channel_info;
mod edge_counter;
mod effect_info;
mod generator_info;

use channel_info::*;
use edge_counter::*;
use effect_info::*;
use generator_info::*;

/// The mixer is responsible for creating, storing and manipulating mixer channels,
/// and the effects and generators they contain.
///
/// The mixer arrangement looks like this:
///
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
    graph: Graph,

    pub edge_counter: EdgeCounter,

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
    pub fn new(project: &Project) -> Self {
        let mut graph = make_graph();

        let channels: Vec<ChannelInfo> = (0..project.mixer.channels.len())
            .map(|channel_index| ChannelInfo::new(&mut graph, project, channel_index))
            .collect();

        let main_buffer = graph.add_node(make_node(BufferNode::default()));
        let main_sum = graph.add_node(make_node(Sum));
        let main_amp = graph.add_node(make_node(AmpNode::new_main()));

        Self {
            graph,
            edge_counter: EdgeCounter::default(),
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
        self.graph.clear_edges();
        self.edge_counter.reset();

        self.edge_counter.add_edge(
            &mut self.graph,
            self.main_buffer,
            self.main_sum,
            EdgeKey::MainBufToMainSum,
        );

        let inputs: Vec<_> = self
            .channels
            .iter()
            .map(|channel| channel.input_node)
            .collect();

        for (channel_index, channel) in self.channels.iter().enumerate() {
            channel.add_edges(&mut self.graph, &mut self.edge_counter);

            // Only add the main channel to the main output sum node.
            // Other channels need to be routed via main.
            if channel_index == 0 {
                self.edge_counter.add_edge(
                    &mut self.graph,
                    channel.output_node,
                    self.main_sum,
                    EdgeKey::MixOutToMainSum,
                );
            } else {
                channel.route_to_inputs(&mut self.graph, &mut self.edge_counter, &inputs);
            }
        }

        self.edge_counter.add_edge(
            &mut self.graph,
            self.main_sum,
            self.main_amp,
            EdgeKey::MainSumToMainAmp,
        );
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
                    let (IndexField::Effect(from), IndexField::Effect(to)) = (from_field, to_field)
                    else {
                        panic!("Action should have only received Effect IndexFields.")
                    };
                    self.channels[*mixer_index].move_effect(*from, *to);
                    true
                }
                Action::DeleteChild(IndexField::Effect(effect_index)) => {
                    self.channels[*mixer_index].delete_effect(&mut self.graph, *effect_index);
                    true
                }
                Action::AddChild(TypeField::Effect(effect)) => {
                    let selector =
                        EffectSelector(*mixer_index, self.channels[*mixer_index].effects_count());
                    self.channels[*mixer_index].add_effect(&mut self.graph, effect, &selector);
                    true
                }
                _ => false,
            },
            Selector::Generator(generator_index) => match action {
                Action::SetIndex(IndexField::Mixer(mixer_channel)) => {
                    let selector = GeneratorSelector(*generator_index);

                    // Find the channel containing this generator, then move it to the correct
                    // channel.
                    for channel in self.channels.iter_mut() {
                        if let Some(generator) = channel.soft_delete_generator(selector) {
                            self.channels[*mixer_channel].soft_add_generator(&generator);
                            break;
                        }
                    }
                    true
                }
                _ => false,
            },
            Selector::MixerMatrixCell(..) => match action {
                // If the matrix changes, reset the routes for each node, then refresh the edges.
                // NOTE: in future, consider what happens if the size of the matrix changes too.
                Action::SetFloat(FloatField::ModFactor, _) => {
                    for (channel_index, channel) in self.channels.iter_mut().enumerate() {
                        channel.refresh_routes(&mut self.graph, channel_index, &store.project);
                    }
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

    /// Returns the buffers corresponding to the output node, which are filled after a processing
    /// run.
    pub fn output_buffers(&self) -> &[Buffer] {
        &self.graph.node_weight(self.main_amp).unwrap().buffers
    }

    /// Processes the graph.
    /// Exposed as a method so that we don't ever have to expose a mutable version of Graph.
    /// Therefore, the mixer is the only object allowed to mutate the Graph.
    pub fn process(&mut self, processor: &mut Processor, payload: &ProcessContext) {
        processor.process(&mut self.graph, payload, self.main_amp);
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
        AdsrEnvelope, AntiAliasingMode, DelayConfig, Effect, EffectInstance, EffectMeta, Generator,
        GeneratorInstance, GeneratorMeta, MixerChannel, SimpleWaveConfig, WaveType,
    };
    use std::collections::HashMap;

    #[test]
    fn one_generator_one_effect() {
        let mut project = Project::default();
        project.generators.push(some_generator());
        project.mixer.channels.push(MixerChannel {
            volume: 1.0,
            effects: vec![some_effect()],
        });

        let mixer = Mixer::new(&project);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeKey::GenToMixIn, 1);
        edge_counts.insert(EdgeKey::MixInToEff, 1);
        edge_counts.insert(EdgeKey::MixInToEffMix, 1);
        edge_counts.insert(EdgeKey::EffToEffMix, 1);
        edge_counts.insert(EdgeKey::EffMixToMixOut, 1);
        edge_counts.insert(EdgeKey::MixOutToMainSum, 1);
        edge_counts.insert(EdgeKey::MainBufToMainSum, 1);
        edge_counts.insert(EdgeKey::MainSumToMainAmp, 1);

        // Main sum and amp nodes (2) +
        // Effect and mixer nodes (2) +
        // Channel input and output nodes (2) +
        // Generator nodes (1) +
        // Buffer nodes (1).
        assert_eq!(mixer.graph.node_count(), 8);
        for (key, count) in mixer.edge_counter.counts.iter() {
            assert_eq!(
                Some(count),
                edge_counts.get(key),
                "{:?} {} {:?}",
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
            .extend(vec![some_generator(), some_generator(), some_generator()]);
        project.generators[2].meta.mixer_channel = 1;

        // One effect on channel 0,
        // Two effects on channel 1.
        project.mixer.channels.extend([
            MixerChannel {
                volume: 1.0,
                effects: vec![some_effect()],
            },
            MixerChannel {
                volume: 1.0,
                effects: vec![some_effect(), some_effect()],
            },
        ]);

        let mixer = Mixer::new(&project);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeKey::GenToMixIn, 3);
        edge_counts.insert(EdgeKey::MixInToEff, 2);
        edge_counts.insert(EdgeKey::MixInToEffMix, 2);
        edge_counts.insert(EdgeKey::EffToEffMix, 3);
        edge_counts.insert(EdgeKey::EffMixToNextEff, 1);
        edge_counts.insert(EdgeKey::EffMixToNextEffMix, 1);
        edge_counts.insert(EdgeKey::EffMixToMixOut, 2);
        edge_counts.insert(EdgeKey::MixOutToMainSum, 1);
        edge_counts.insert(EdgeKey::MainBufToMainSum, 1);
        edge_counts.insert(EdgeKey::MainSumToMainAmp, 1);

        // Main sum and amp nodes (2) +
        // Effect and mixer nodes (2 * 3 effects) +
        // Channel input and output nodes (2 * 2 channels) +
        // Generator nodes (3) +
        // Buffer nodes (1).
        assert_eq!(mixer.graph.node_count(), 16);
        for (key, count) in mixer.edge_counter.counts.iter() {
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
        let config = SimpleWaveConfig {
            wave: WaveType::Sine,
            envelope: AdsrEnvelope {
                attack: 0.1,
                decay: 0.1,
                sustain: 0.8,
                release: 0.1,
            },
            osc_count: 4,
            detune_cents: 5.0,
            anti_aliasing_mode: AntiAliasingMode::Off,
            oversample_factor: 2,
        };

        let meta = GeneratorMeta {
            volume: 1.0,
            mute: false,
            pan: 0.0,
            mixer_channel: 0,
        };

        GeneratorInstance {
            it: Generator::SimpleWave(config),
            meta,
        }
    }

    // TODO: write tests for updating the store with actions.
    // TODO: write tests for routing between mixer channels.
}
