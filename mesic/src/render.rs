use crate::SAMPLE_RATE;
use crate::wave::polyphonic_wave;
use shared::model::{GeneratorType, Project};
use crate::sig::{Graph, Processor, BufferNode};
use dasp_graph::NodeData;
use dasp_signal::Signal;
use petgraph::stable_graph::NodeIndex;

pub fn render(project: &Project) -> Vec<f32> {
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
    let mut total_wave: Vec<f32> = vec![0.0; track_samples];

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

        let offset_samples =
            (Into::<f32>::into(note.offset) / bpm * 60.0 * SAMPLE_RATE as f32) as usize;
        wave.iter().enumerate().for_each(|(i, value)| {
            total_wave[i + offset_samples] += value;
        })
        // TODO: account for offsets properly, instead of just appending here.
    }

    // Create a graph and a processor with some suitable capacity to avoid dynamic allocation.
    let max_nodes = 1024;
    let max_edges = 1024;
    let mut g = Graph::with_capacity(max_nodes, max_edges);
    let mut p = Processor::with_capacity(max_nodes);

    // Add some nodes and edges...
    let node = g.add_node(NodeData::new1(BufferNode { buffer: total_wave.clone(), index: 0 }));

    // Process all nodes within the graph that output to the node at `n_id`.
    let mut output: Vec<f32> = vec![];
    let process_count = total_wave.len() / 64 + 1;
    for _ in 0..process_count {
        p.process(&mut g, node);
        // TODO: optimize.
        let vec: Vec<_> = g.node_weight(node).unwrap().buffers[0].iter().collect();
        output.extend(vec);
    }

    output

    // TODO before submit: add effects back
    //apply_effects(wave_node, &mixer_channel.effects)
}
