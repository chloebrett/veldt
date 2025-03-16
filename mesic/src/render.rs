use crate::SAMPLE_RATE;
use crate::effect::apply_effects;
use crate::wave::polyphonic_wave;
use shared::model::{GeneratorInstance, GeneratorType, MixerChannel, Track};
use shared::types::Beats;

pub fn render(
    track: &Track,
    mixer_channel: &MixerChannel,
    generator: &GeneratorInstance,
    bpm: Beats,
) -> Vec<f32> {
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
        let mut wave = polyphonic_wave(
            &note.note.pitch_name,
            note.note.beats,
            bpm,
            generator.meta.volume,
            &generator_config,
        );

        let offset_samples =
            (Into::<f32>::into(note.offset) / bpm * 60.0 * SAMPLE_RATE as f32) as usize;
        let mut placed_wave: Vec<f32> = vec![0.0; offset_samples];
        placed_wave.append(&mut wave);
        placed_wave.iter().enumerate().for_each(|(i, value)| {
            total_wave[i] += value;
        })
        // TODO: account for offsets properly, instead of just appending here.
    }

    apply_effects(&total_wave, &mixer_channel.effects)
}
