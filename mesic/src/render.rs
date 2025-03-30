use crate::SAMPLE_RATE;
use crate::graph::{BufferNode, RenderGraph, make_graph};
use crate::wave::polyphonic_wave;
use dasp_graph::{BoxedNode, NodeData};
use shared::model::{GeneratorType, Project};

pub fn render(project: &Project) -> RenderGraph {
    let track = &project.tracks[0];
    let generator = &project.generators[0];
    let mixer_channel = &project.mixer[0];
    let bpm = project.bpm;

    let track_beats: f32 = track
        .notes
        .iter()
        .map(|placed_note| Into::<f32>::into(placed_note.offset) + placed_note.note.beats)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    let track_samples = (track_beats / bpm * 60.0 * SAMPLE_RATE as f32) as usize;
    let mut output: Vec<f32> = vec![0.0; track_samples];

    let generator_config = match &generator.kind {
        GeneratorType::SimpleWave { config } => config,
    };

    for note in &track.notes {
        let wave = polyphonic_wave(
            &note.note.pitch_name,
            note.note.beats,
            bpm,
            generator.meta.volume,
            generator_config,
        );

        let offset_samples = *note.offset / bpm * 60.0 * SAMPLE_RATE as f32;
        wave.iter().enumerate().for_each(|(i, value)| {
            output[i + offset_samples as usize] += value;
        })
    }

    let mut graph = make_graph();

    let buffer_node: BufferNode = output.into();
    let buffer_node_index = graph.add_node(NodeData::new1(BoxedNode::new(buffer_node)));

    let mut render_graph = RenderGraph::new(graph, track_samples, buffer_node_index);

    for effect in &mixer_channel.effects {
        render_graph.add_effect_with_mixer(effect.clone());
    }

    render_graph
}
