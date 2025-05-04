use super::make_node;
use crate::graph::{
    Graph, SimpleWaveGeneratorNode, SubSynthNode,
};
use petgraph::stable_graph::NodeIndex;
use shared::model::{Generator, GeneratorInstance, PlacementType, Project};

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
