use crate::SAMPLE_RATE;
use crate::graph::{GeneratorNode, RenderGraph};
use shared::model::Project;

pub fn render(project: &Project) -> RenderGraph {
    let mut render_graph = RenderGraph::default();
    let bpm = project.bpm;
    // TODO account for multiple tracks and placements.
    // This will break with multiple tracks
    for (index, placement) in project.track_placements.iter().enumerate() {
        let mut track = project.tracks[placement.track_id].clone();
        for note in track.notes.iter_mut() {
            note.offset += placement.offset
        }

        // Work out how many samples the graph needs to render.
        let track_length = placement
            .clipped_duration
            .unwrap_or(track.unclipped_duration());
        let track_samples = (*track_length / bpm * 60.0 * SAMPLE_RATE as f32) as usize;
        // Create a generator node.
        // Use multiple generators.
        let generator_index = 0;
        let generator_node =
            GeneratorNode::new(project.generators[generator_index].clone(), track, bpm);
        render_graph.add_generator(generator_node, track_samples);
        // Apply effects.
        let mixer_channel = &project.mixer[0];
        for effect in &mixer_channel.effects {
            render_graph.add_effect_with_mixer(effect.clone(), index);
        }
    }

    render_graph
}
