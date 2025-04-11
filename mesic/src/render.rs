use crate::SAMPLE_RATE;
use crate::graph::{GeneratorNode, RenderGraph, make_graph};
use dasp_graph::{BoxedNodeSend, NodeData};
use shared::model::Project;

pub fn render(project: &Project) -> RenderGraph {
    let track = &project.tracks[0];
    let bpm = project.bpm;

    // Work out how many samples the graph needs to render.
    let track_beats: f32 = track
        .notes
        .iter()
        .map(|placed_note| Into::<f32>::into(placed_note.offset) + placed_note.note.beats)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let track_samples = (track_beats / bpm * 60.0 * SAMPLE_RATE as f32) as usize;

    // Create a generator node, and create a render graph that uses it as the starting point.
    let mut graph = make_graph();
    let generator_node = GeneratorNode::new(project.generators[0].clone(), track.clone(), bpm);
    let generator_node_index = graph.add_node(NodeData::new2(BoxedNodeSend::new(generator_node)));
    let mut render_graph = RenderGraph::new(graph, track_samples, generator_node_index);

    // Apply effects.
    let mixer_channel = &project.mixer[0];
    for effect in &mixer_channel.effects {
        render_graph.add_effect_with_mixer(effect.clone());
    }

    render_graph
}
