use ordered_float::OrderedFloat;

use crate::model::{
    AdsrEnvelope, CompressorConfig, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig,
    EqType, GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, Note, PitchName,
    PlacedNote, Project, Sample, ScaleValue, SimpleWaveConfig, Track, TrackPlacement, WaveType,
};

pub fn make_project() -> Project {
    Project {
        name: "My Project".to_string(),
        tracks: vec![Track {
            notes: vec![PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: ScaleValue::A,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: OrderedFloat(0.0),
            }],
        }],
        track_placements: vec![TrackPlacement {
            track_id: 3,
            start_position: 2.5.into(),
            clipped_duration: Some(5.2.into()),
            visual_placement: 6,
        }],
        samples: vec![Sample {
            data: vec![0.0, 1.0, 3.0],
            sample_rate: 1.0,
        }],
        generators: vec![GeneratorInstance {
            id: 0,
            kind: GeneratorType::SimpleWave {
                config: SimpleWaveConfig {
                    wave: WaveType::Sine,
                    envelope: AdsrEnvelope {
                        attack: 0.1,
                        decay: 0.1,
                        sustain: 0.8,
                        release: 0.1,
                    },
                    osc_count: 4,
                    detune_cents: 5.0,
                },
            },
            meta: GeneratorMeta { volume: 1.0 },
        }],
        mixer: vec![MixerChannel {
            effects: vec![
                EffectInstance {
                    effect: Effect::SimpleEq {
                        config: EqConfig {
                            kind: EqType::SimpleResonator,
                            fc: 1000.0,
                            q: 1.0,
                        },
                    },
                    meta: EffectMeta { id: 0, wet: 1.0 },
                },
                EffectInstance {
                    effect: Effect::SimpleDelay {
                        config: DelayConfig { delay_ms: 250.0 },
                    },
                    meta: EffectMeta { id: 1, wet: 0.5 },
                },
                EffectInstance {
                    effect: Effect::SimpleCompressor {
                        config: CompressorConfig {
                            threshold: 3.2,
                            attack_ms: 20.0,
                            release_ms: 500.0,
                            ratio: 3.0,
                            gain: 1.0,
                        },
                    },
                    meta: EffectMeta { id: 2, wet: 1.0 },
                },
            ],
        }],
        bpm: 120.0,
    }
}
