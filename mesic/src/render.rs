use crate::graph::{GeneratorNode, RenderGraph};
use shared::model::Project;

pub fn render(project: &Project) -> RenderGraph {
    let mut render_graph = RenderGraph::default();
    let bpm = project.bpm;
    // Add tracks with a placement to graph.
    // TODO: each generator should play all tracks it's linked to - we don't need a different
    // generator for every track!
    for (index, placement) in project.track_placements.iter().enumerate() {
        let track = project.tracks[placement.track_id as usize].clone();

        // Create a generator node.
        // TODO use multiple generators.
        let generator_index = 0;
        let generator_node = GeneratorNode::new(
            project.generators[generator_index].clone(),
            track,
            placement.clone(),
            bpm,
        );
        render_graph.add_generator(generator_node);

        // Apply effects.
        // TODO allow multiple effects
        let mixer_channel = &project.mixer[0];
        for effect in &mixer_channel.effects {
            render_graph.add_generator_effect_with_mixer(effect.clone(), index);
        }
    }

    render_graph
}
