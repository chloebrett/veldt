use crate::effect::apply_effects;
use crate::sig::sum;
use crate::wave::{SupersawConfig, polyphonic_wave};
use shared::model::{
    BandPassAlgorithm, Effect, EffectInstance, EffectMeta, EqType, MixerChannel, PassType, Track,
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
            amplitude: 0.5,

            delay_ms: 250.0,
        },
        meta: EffectMeta { id: 0, wet: 0.5 },
    };
    let simple_resonator = EffectInstance {
        effect: Effect::SimpleEq {
            kind: EqType::Pass {
                kind: PassType::Band {
                    algorithm: BandPassAlgorithm::SimpleResonator,
                },
            },

            freq: resonant_freq,

            q_value: resonance_q, // demonstrative range: 1.0 to 10.0 - but can go lower or higher.
        },
        meta: EffectMeta {
            id: 1,
            wet: resonance_wet,
        },
    };
    let effects = vec![delay, simple_resonator];
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
