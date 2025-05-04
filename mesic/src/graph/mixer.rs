use super::{
    AmpNode, CompressorNode, DelayNode, EqNode, Graph, MixerNode, ModDelayNode, ProcessContext,
    SimpleWaveGeneratorNode, SubSynthNode, make_graph,
};
use dasp_graph::{BoxedNodeSend, Node, NodeData, node::Sum};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Effect, EffectInstance, Generator, GeneratorInstance, PlacementType, Project};
use state::EffectSelector;

type ChannelIndex = usize;

struct GeneratorInfo {
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

struct EffectInfo {
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
        graph.add_edge(self.mixer_node, next_effect.mixer_node(), ());
    }
}

struct ChannelInfo {
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
        }

        let effects: Vec<EffectInfo> = project.mixer[channel_index]
            .effects
            .iter()
            .enumerate()
            .map(|(effect_index, effect)| {
                EffectInfo::new(graph, effect, &EffectSelector(channel_index, effect_index))
            })
            .collect();

        for (effect, next_effect) in effects.iter().zip(effects.iter().next()) {
            effect.link_to(next_effect, graph);
        }

        // TODO: wire up the amp node to read the correct volume.
        let output_node = graph.add_node(make_node(AmpNode::default()));

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
    pub fn from_project(project: &Project) -> Self {
        let mut graph = make_graph();

        let main_sum = graph.add_node(make_node(Sum));

        let channels: Vec<ChannelInfo> = (0..project.mixer.len())
            .map(|channel_index| ChannelInfo::new(&mut graph, project, channel_index))
            .collect();

        for channel in &channels {
            graph.add_edge(channel.output_node, main_sum, ());
        }

        let main_amp = graph.add_node(make_node(AmpNode::default()));
        graph.add_edge(main_sum, main_amp, ());

        Self {
            graph,
            channels,
            main_sum,
            main_amp,
        }
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
