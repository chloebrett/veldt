use crate::SAMPLE_RATE;
use crate::Sig;
use crate::effect::apply_effects;
use crate::sig::OutputNode;
use crate::wave::polyphonic_wave;
use shared::model::{GeneratorType, Project};
use std::cell::RefCell;
use std::rc::Rc;

pub fn render(project: &Project) -> Rc<RefCell<dyn Sig>> {
    let track = &project.tracks[0];
    let generator = &project.generators[0];
    let mixer_channel = &project.mixer[0];
    let bpm = project.bpm;

    let track_beats: f32 = track
        .notes
        .iter()
        .map(|placed_note| Into::<f32>::into(placed_note.offset) + placed_note.note.beats)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();
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

    let wave_node = Rc::new(RefCell::new(OutputNode {
        buffer: total_wave,
        index: 0,
    }));

    apply_effects(wave_node, &mixer_channel.effects)
}
