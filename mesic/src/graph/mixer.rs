use super::{
    AmpNode, BufferNode, CompressorNode, DelayNode, EqNode, Graph, MixerNode, ModDelayNode,
    ProcessContext, Processor, SimpleWaveGeneratorNode, SubSynthNode, make_graph,
};
use dasp_frame::Stereo;
use dasp_graph::{BoxedNodeSend, Buffer, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance, Generator, GeneratorInstance, PlacementType, Project};
use state::EffectSelector;

#[expect(dead_code)] // Will need to read fields to manipulate later.
pub struct GeneratorInfo {
    // Generator index within the project model.
    generator_index: usize,

    node: NodeIndex,
}

impl GeneratorInfo {
    pub fn new(
        graph: &mut Graph,
        project: &Project,
        generator: &GeneratorInstance,
        generator_index: usize,
    ) -> Self {
        let bpm = project.bpm;
        // Placements that are linked to this generator.
        let placements: Vec<_> = project
            .placements
            .clone()
            .into_iter()
            .filter(|it| match &it.kind {
                PlacementType::Track(it) => it.generator_index == generator_index,
                _ => false,
            })
            .collect();

        // TODO: do better than just cloning all the tracks!
        // Perhaps load the relevant track data from the store
        // out of the payload in each processing cycle?
        // Or even just get a &[Track] containing all the tracks in the store, in each
        // processing cycle, and don't store it anywhere.
        let tracks = project.tracks.clone();

        let node = match &generator {
            GeneratorInstance {
                it: Generator::SimpleWave(config),
                meta,
            } => make_node(SimpleWaveGeneratorNode::new(
                config.clone(),
                meta.clone(),
                generator_index,
                placements,
                tracks,
                bpm,
            )),
            GeneratorInstance {
                it: Generator::SubSynth(config),
                meta,
            } => make_node(SubSynthNode::new(
                config.clone(),
                meta.clone(),
                generator_index,
                placements,
                tracks,
                bpm,
            )),
            _ => {
                // TODO: support adding other types of generators to the graph.
                panic!("Not yet implemented.")
            }
        };

        let node = graph.add_node(node);

        Self {
            generator_index,
            node,
        }
    }

    pub fn node(&self) -> NodeIndex {
        self.node
    }
}

#[expect(dead_code)] // Will need to read fields to manipulate later.
#[derive(Debug)]
pub struct EffectInfo {
    // Effect index within the project model.
    effect_index: usize,

    effect_node: NodeIndex,
    mixer_node: NodeIndex,
}

impl EffectInfo {
    pub fn new(graph: &mut Graph, effect: &EffectInstance, effect_sel: &EffectSelector) -> Self {
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
        log::info!("Added edge: effect -> effect mixer");
        graph.add_edge(effect_node, mixer_node, ());

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

    pub fn link_to(&self, next_effect: &EffectInfo, graph: &mut Graph) {
        // TODO: confirm this results in the correct direction for wet/dry.
        graph.add_edge(self.mixer_node, next_effect.effect_node(), ());
        log::info!("Added edge: effect mixer -> next effect");
        graph.add_edge(self.mixer_node, next_effect.mixer_node(), ());
        log::info!("Added edge: effect mixer -> next effect mixer");
    }
}

#[expect(dead_code)] // Will need to read fields to manipulate later.
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
    output_node: NodeIndex,
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

        for generator in &generators {
            graph.add_edge(generator.node(), input_node, ());
            log::info!("Added edge: generator -> mixer channel input");
        }

        let effects: Vec<EffectInfo> = project.mixer[channel_index]
            .effects
            .iter()
            .enumerate()
            .map(|(effect_index, effect)| {
                EffectInfo::new(graph, effect, &EffectSelector(channel_index, effect_index))
            })
            .collect();

        // TODO: get .zip() working.
        if !effects.is_empty() {
            for i in 0..effects.len() - 1 {
                let effect = &effects[i];
                let next_effect = &effects[i + 1];
                effect.link_to(next_effect, graph);
            }
        }

        // TODO: wire up the amp node to read the correct volume.
        let output_node = graph.add_node(make_node(AmpNode::default()));

        // Link up the input -> effects -> output.
        // If there are no effects, link directly from input -> output.
        if effects.is_empty() {
            graph.add_edge(input_node, output_node, ());
            log::info!("Added edge: mixer channel input -> mixer channel output");
        } else {
            let first = effects.first().unwrap();
            let last = effects.last().unwrap();

            // TODO: check the wet/dry direction here.
            graph.add_edge(input_node, first.effect_node, ());
            log::info!("Added edge: mixer channel input -> first effect");
            graph.add_edge(input_node, first.mixer_node, ());
            log::info!("Added edge: mixer channel input -> first effect mixer");

            graph.add_edge(last.mixer_node, output_node, ());
            log::info!("Added edge: last effect mixer -> mixer channel output");
        }

        Self {
            generators,
            input_node,
            effects,
            output_node,
        }
    }
}

/// Mixer arrangement looks like this:
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

        let main_sum = graph.add_node(make_node(Sum));
        let main_amp = graph.add_node(make_node(AmpNode::default()));
        graph.add_edge(main_sum, main_amp, ());
        log::info!("Added edge: main sum -> main amp");

        Self {
            graph,
            channels: vec![],
            main_sum,
            main_amp,
        }
    }

    pub fn from_project(project: &Project) -> Self {
        let mut graph = make_graph();

        let main_sum = graph.add_node(make_node(Sum));

        // TODO: always have channel 0 as the "main" channel?
        // How would this change the graph?
        let channels: Vec<ChannelInfo> = (0..project.mixer.len())
            .map(|channel_index| ChannelInfo::new(&mut graph, project, channel_index))
            .collect();

        for channel in &channels {
            graph.add_edge(channel.output_node, main_sum, ());
            log::info!("Added edge: mixer channel output -> main sum");
        }

        let main_amp = graph.add_node(make_node(AmpNode::default()));
        graph.add_edge(main_sum, main_amp, ());
        log::info!("Added edge: main sum -> main amp");

        Self {
            graph,
            channels,
            main_sum,
            main_amp,
        }
    }

    pub fn from_audio(audio: &Vec<Stereo<f32>>) -> Self {
        let mut graph = make_graph();

        // TODO: simply give buffers their own dedicated mixer channel,
        // and then otherwise don't treat them differently to other generators.
        // This way, the user can run effects on samples, etc.
        // There is a bit more thinking to be done about how the "playing audio as a preview" idea
        // should work anyway.
        let buffer_node: BufferNode = audio.clone().into();
        let main_buffer = graph.add_node(make_node(buffer_node));

        let main_sum = graph.add_node(make_node(Sum));
        let main_amp = graph.add_node(make_node(AmpNode::default()));

        graph.add_edge(main_buffer, main_sum, ());
        log::info!("Added edge: main buffer -> main sum");
        graph.add_edge(main_sum, main_amp, ());
        log::info!("Added edge: main sum -> main amp");

        Self {
            graph,
            channels: vec![],
            main_sum,
            main_amp,
        }
    }

    pub fn output_buffers(&self) -> &[Buffer] {
        &self.graph.node_weight(self.main_amp).unwrap().buffers
    }

    /// Processes the graph.
    /// Exposed as a method so that we don't ever have to expose a mutable version of Graph.
    /// Therefore, the mixer is the only object allowed to mutate the Graph.
    pub fn process(&mut self, processor: &mut Processor, payload: &ProcessContext) {
        processor.process(&mut self.graph, payload, self.main_amp);
    }

    pub fn channels(&self) -> &Vec<ChannelInfo> {
        &self.channels
    }

    pub fn output(&self) -> NodeIndex {
        self.main_amp
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
        AdsrEnvelope, AntiAliasingMode, DelayConfig, EffectMeta, GeneratorMeta, MixerChannel,
        SimpleWaveConfig, WaveType,
    };

    #[test]
    fn empty_mixer() {
        let empty_project = Project::default();
        let mixer = Mixer::from_project(&empty_project);

        // Main sum and amp nodes (2)
        assert_eq!(mixer.graph.node_count(), 2);
        // Main sum -> main amp (1)
        assert_eq!(mixer.graph.edge_count(), 1);
        assert_eq!(mixer.channels().len(), 0);
    }

    #[test]
    fn one_generator_one_effect() {
        let mut project = Project::default();
        project.generators.push(some_generator());
        project.mixer.push(MixerChannel {
            effects: vec![some_effect()],
        });

        let mixer = Mixer::from_project(&project);

        // Main sum and amp nodes (2) +
        // Effect and mixer nodes (2) +
        // Channel input and output nodes (2) +
        // Generator nodes (1)
        assert_eq!(mixer.graph.node_count(), 7);
        // Generator -> mixer input (1)
        // Mixer input -> effect (1)
        // Mixer input -> wet/dry mixer (1)
        // Effect -> wet/dry mixer (1)
        // Wet/dry mixer -> mixer output (1)
        // Mixer output -> main sum (1)
        // Main sum -> main amp (1)
        assert_eq!(mixer.graph.edge_count(), 7);
        assert_eq!(mixer.channels().len(), 1);
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
        assert_eq!(mixer.graph.edge_count(), 15);
        assert_eq!(mixer.channels().len(), 2);
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
