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
    let mut total_wave: Vec<f32> = vec![];

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

        // TODO: account for offsets properly, instead of just appending here.
        total_wave.append(&mut wave);
    }

    apply_effects(&total_wave, &mixer_channel.effects)
}
