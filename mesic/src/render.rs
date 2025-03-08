use crate::effect::apply_effects;
use crate::sig::sum;
use crate::wave::{SupersawConfig, polyphonic_wave};
use shared::model::{
    Effect, EffectInstance, EffectMeta, EqType, MixerChannel, Track, DelayConfig, EqConfig,
};
use shared::types::{Freq, KnobPosition};

pub fn render(
    track: &Track,
    supersaw_config: SupersawConfig,
    resonant_freq: Freq,
    resonance_q: KnobPosition,
    resonance_wet: KnobPosition,
) -> Vec<f32> {
    let bpm = track.bpm;
    let mut total_wave: Vec<f32> = vec![];

    let delay = EffectInstance {
        effect: Effect::SimpleDelay {
            config: DelayConfig {
                amplitude: 0.5,

                delay_ms: 250.0,
            },
        },
        meta: EffectMeta { id: 0, wet: 0.5 },
    };
    let eq = EffectInstance {
        effect: Effect::SimpleEq {
            config: EqConfig {
                kind: EqType::SimpleSecondOrderBandStop,

                fc: resonant_freq,

                q: resonance_q,
            },
        },
        meta: EffectMeta {
            id: 1,
            wet: resonance_wet,
        },
    };
    let effects = vec![delay, eq];
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
