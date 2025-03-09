use crate::effect::apply_effects;
use crate::sig::sum;
use crate::wave::{SupersawConfig, polyphonic_wave};
use shared::model::{EffectInstance, MixerChannel, Track};

pub fn render(
    track: &Track,
    supersaw_config: SupersawConfig,
    effects: Vec<EffectInstance>,
) -> Vec<f32> {
    let bpm = track.bpm;
    let mut total_wave: Vec<f32> = vec![];

    let mixer_channel = MixerChannel { effects };

    for sequence in &track.sequences {
        // TODO: use the offset value instead of ignoring.
        let synth = &track.synths.get(sequence.synth_index).unwrap();
        println!("{:?}", sequence);

        let mut sequence_wave: Vec<f32> = vec![];
        for note in &sequence.notes {
            println!("{:?}", note);
            let mut wave = polyphonic_wave(
                &note.pitch_name,
                note.beats,
                bpm,
                sequence.volume * synth.volume,
                &synth.envelope,
                synth.wave,
                supersaw_config.clone(),
            );
            sequence_wave.append(&mut wave);
        }

        total_wave = sum(total_wave, sequence_wave);
    }

    apply_effects(total_wave, mixer_channel.effects)
}
