use crate::graph::{AmpNode, BufferNode, Graph, ProcessContext, Processor, make_graph};
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::Project;
use std::collections::HashMap;

mod channel_info;
mod effect_info;
mod generator_info;

use channel_info::*;
use effect_info::*;
use generator_info::*;

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

// Counts edges in the graph by type.
// Helpful for testing/debugging.
// Note: consider extending this to also count nodes, if that would be helpful.
#[derive(Debug, Default)]
pub struct EdgeCounter {
    counts: HashMap<EdgeKey, usize>,
}

impl EdgeCounter {
    fn add_edge(&mut self, graph: &mut Graph, from: NodeIndex, to: NodeIndex, key: EdgeKey) {
        graph.add_edge(from, to, ());
        let current = self.counts.get(&key).unwrap_or(&0);
        self.counts.insert(key, current + 1);
        log::info!("Added edge: {:?}", key);
    }
}

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
/// s // main sum (joins together all channels)
/// |
/// v
/// a // main out
///
/// ------------
/// a = amp node
/// e = effect node
/// m = mixer node
/// g = generator node
/// s = sum node
#[expect(dead_code)] // Will need to read fields to manipulate later.
pub struct Mixer {
    graph: Graph,

    pub edge_counter: EdgeCounter,

    channels: Vec<ChannelInfo>,

    // Main sum node.
    // Adds up all of the mixer channel outputs.
    // Directs its output to the main_amp node.
    main_sum: NodeIndex,

    // Main output amp node.
    main_amp: NodeIndex,
}

impl Mixer {
    pub fn empty() -> Self {
        let mut graph = make_graph();
        let mut edge_counter = EdgeCounter::default();

        let main_sum = graph.add_node(make_node(Sum));
        let main_amp = graph.add_node(make_node(AmpNode::default()));
        edge_counter.add_edge(&mut graph, main_sum, main_amp, EdgeKey::MainSumToMainAmp);

        Self {
            graph,
            edge_counter,
            channels: vec![],
            main_sum,
            main_amp,
        }
    }

    pub fn from_project(project: &Project) -> Self {
        let mut graph = make_graph();
        let mut edge_counter = EdgeCounter::default();

        let main_sum = graph.add_node(make_node(Sum));

        // TODO: always have channel 0 as the "main" channel?
        // How would this change the graph?
        let channels: Vec<ChannelInfo> = (0..project.mixer.len())
            .map(|channel_index| {
                ChannelInfo::new(&mut graph, &mut edge_counter, project, channel_index)
            })
            .collect();

        for channel in &channels {
            edge_counter.add_edge(
                &mut graph,
                channel.output_node,
                main_sum,
                EdgeKey::MixOutToMainSum,
            );
        }

        let main_amp = graph.add_node(make_node(AmpNode::default()));
        graph.add_edge(main_sum, main_amp, ());
        edge_counter.add_edge(&mut graph, main_sum, main_amp, EdgeKey::MainSumToMainAmp);

        Self {
            graph,
            edge_counter,
            channels,
            main_sum,
            main_amp,
        }
    }

    pub fn from_audio(audio: &Vec<Stereo<f32>>) -> Self {
        let mut graph = make_graph();
        let mut edge_counter = EdgeCounter::default();

        // TODO: simply give buffers their own dedicated mixer channel,
        // and then otherwise don't treat them differently to other generators.
        // This way, the user can run effects on samples, etc.
        // There is a bit more thinking to be done about how the "playing audio as a preview" idea
        // should work anyway.
        let buffer_node: BufferNode = audio.clone().into();
        let main_buffer = graph.add_node(make_node(buffer_node));

        let main_sum = graph.add_node(make_node(Sum));
        let main_amp = graph.add_node(make_node(AmpNode::default()));

        edge_counter.add_edge(&mut graph, main_buffer, main_sum, EdgeKey::MainBufToMainSum);
        edge_counter.add_edge(&mut graph, main_sum, main_amp, EdgeKey::MainSumToMainAmp);

        Self {
            graph,
            edge_counter,
            channels: vec![],
            main_sum,
            main_amp,
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

    #[test]
    fn empty_mixer() {
        let empty_project = Project::default();
        let mixer = Mixer::from_project(&empty_project);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeKey::MainSumToMainAmp, 1);

        // Main sum and amp nodes (2)
        assert_eq!(mixer.graph.node_count(), 2);
        // Main sum -> main amp (1)
        assert_eq!(mixer.edge_counter.counts, edge_counts);
        assert_eq!(mixer.channels.len(), 0);
    }

    #[test]
    fn one_generator_one_effect() {
        let mut project = Project::default();
        project.generators.push(some_generator());
        project.mixer.push(MixerChannel {
            effects: vec![some_effect()],
        });

        let mixer = Mixer::from_project(&project);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeKey::GenToMixIn, 1);
        edge_counts.insert(EdgeKey::MixInToEff, 1);
        edge_counts.insert(EdgeKey::MixInToEffMix, 1);
        edge_counts.insert(EdgeKey::EffToEffMix, 1);
        edge_counts.insert(EdgeKey::EffMixToMixOut, 1);
        edge_counts.insert(EdgeKey::MixOutToMainSum, 1);
        edge_counts.insert(EdgeKey::MainSumToMainAmp, 1);

        // Main sum and amp nodes (2) +
        // Effect and mixer nodes (2) +
        // Channel input and output nodes (2) +
        // Generator nodes (1)
        assert_eq!(mixer.graph.node_count(), 7);
        assert_eq!(mixer.edge_counter.counts, edge_counts);
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
        project.mixer.extend([
            MixerChannel {
                effects: vec![some_effect()],
            },
            MixerChannel {
                effects: vec![some_effect(), some_effect()],
            },
        ]);

        let mixer = Mixer::from_project(&project);

        let mut edge_counts = HashMap::new();
        edge_counts.insert(EdgeKey::GenToMixIn, 3);
        edge_counts.insert(EdgeKey::MixInToEff, 2);
        edge_counts.insert(EdgeKey::MixInToEffMix, 2);
        edge_counts.insert(EdgeKey::EffToEffMix, 3);
        edge_counts.insert(EdgeKey::EffMixToNextEff, 1);
        edge_counts.insert(EdgeKey::EffMixToNextEffMix, 1);
        edge_counts.insert(EdgeKey::EffMixToMixOut, 2);
        edge_counts.insert(EdgeKey::MixOutToMainSum, 2);
        edge_counts.insert(EdgeKey::MainSumToMainAmp, 1);

        // Main sum and amp nodes (2) +
        // Effect and mixer nodes (2 * 3 effects) +
        // Channel input and output nodes (2 * 2 channels) +
        // Generator nodes (3).
        assert_eq!(mixer.graph.node_count(), 15);
        // Generator -> mixer input (3)
        // Mixer input -> effect (2)
        // Mixer input -> wet/dry mixer (2)
        // Effect -> wet/dry mixer (3)
        // Wet/dry mixer -> mixer output (2)
        // Mixer output -> main sum (2)
        // Main sum -> main amp (1)
        assert_eq!(mixer.edge_counter.counts, edge_counts);
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
}
