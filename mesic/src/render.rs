use crate::SAMPLE_RATE;
use crate::graph::{GeneratorNode, RenderGraph, make_graph};
use dasp_graph::{BoxedNode, NodeData};
use shared::model::Project;

pub fn render(project: &Project) -> RenderGraph {
    let mut graph = make_graph();

    let track = project.tracks[0].clone();
    let bpm = project.bpm;
    let generator_node = GeneratorNode::new(project.generators[0].clone(), track.clone(), bpm);
    let generator_node_index = graph.add_node(NodeData::new1(BoxedNode::new(generator_node)));

    let track_beats: f32 = track
        .notes
        .iter()
        .map(|placed_note| {
            let offset: f32 = placed_note.offset.into();
            offset + placed_note.note.beats
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let track_samples = (track_beats / bpm * 60.0 * SAMPLE_RATE as f32) as usize;
    let mut render_graph = RenderGraph::new(graph, track_samples, generator_node_index);

    let mixer_channel = &project.mixer[0];
    for effect in &mixer_channel.effects {
        render_graph.add_effect_with_mixer(effect.clone());
    }

    render_graph
}
