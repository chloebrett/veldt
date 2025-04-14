use crate::SAMPLE_RATE;
use crate::graph::{GeneratorNode, RenderGraph, make_graph};
use dasp_graph::{BoxedNodeSend, NodeData};
use shared::model::Project;

pub fn render(project: &Project) -> RenderGraph {
    // TODO account for multiple tracks and placements.
    // This will break with multiple tracks
    let track_index = 0;
    let placement_index = 0;
    let mut track = project.tracks[track_index].clone();
    for note in track.notes.iter_mut() {
        note.offset += project.track_placements[placement_index].offset
    }

    let bpm = project.bpm;

    // Work out how many samples the graph needs to render.
    let track_length = project.track_placements[placement_index]
        .clipped_duration
        .unwrap_or(track.find_max_note_length());
    let track_samples = (*track_length / bpm * 60.0 * SAMPLE_RATE as f32) as usize;

    // Create a generator node, and create a render graph that uses it as the starting point.
    let mut graph = make_graph();
    let generator_node = GeneratorNode::new(project.generators[0].clone(), track, bpm);
    let generator_node_index = graph.add_node(NodeData::new2(BoxedNodeSend::new(generator_node)));
    let mut render_graph = RenderGraph::new(graph, track_samples, generator_node_index);

    // Apply effects.
    let mixer_channel = &project.mixer[0];
    for effect in &mixer_channel.effects {
        render_graph.add_effect_with_mixer(effect.clone());
    }

    render_graph
}
