use ordered_float::OrderedFloat;
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, CompressorConfig, DelayConfig, Effect, EffectInstance,
    EffectMeta, EqConfig, EqType, GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel,
    ModDelayConfig, ModMatrix, Note, PitchName, PlacedNote, Project, Scale, ScaleValue,
    SimpleWaveConfig, Track, TrackPlacement, WaveType, SubSynthConfig, OscillatorConfig, FileTreeConfig, FilenameTree
};
use shared::types::Volume;

#[derive(Debug)]
pub struct StoreData {
    pub project: Project,
    pub key: ScaleValue,
    pub scale: Scale,
    pub volume: Volume,
    pub sample_tree: Option<FilenameTree>,
    pub sample_tree_config: FileTreeConfig,
    pub project_list: Vec<String>,
    pub load_project_name: Option<String>,
}

impl Default for StoreData {
    fn default() -> Self {
        StoreData {
            project: Project {
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
                    offset: OrderedFloat(0.0),
                }],
                track_placements: vec![TrackPlacement {
                    track_id: 0,
                    clipped_duration: None,
                    offset: OrderedFloat(0.0),
                    visual_placement: 0,
                }],
                samples: vec![],
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
                            anti_aliasing_mode: AntiAliasingMode::Off,
                            oversample_factor: 2,
                        },
                    },
                    meta: GeneratorMeta {
                        volume: 1.0,
                        mute: false,
                        pan: 0.0,
                    },
                },
                GeneratorInstance {
                    id: 1,
                    kind: GeneratorType::SubSynth {
                        config: SubSynthConfig{
                            oscillators: [
                            OscillatorConfig{
                                wave: WaveType::Sine,
                                volume: 1.0,
                                pan: 0.0,
                                coarse_detune: 0.0,
                                fine_detune: 0.0,
                            }, 
                            OscillatorConfig{
                                wave: WaveType::Saw,
                                volume: 1.0,
                                pan: 0.0,
                                coarse_detune: 0.0,
                                fine_detune: 0.0,
                            },
                            OscillatorConfig{
                                wave: WaveType::Square,
                                volume: 1.0,
                                pan: 0.0,
                                coarse_detune: 0.0,
                                fine_detune: 0.0,
                            }]
                        }
                    },
                    meta: GeneratorMeta {
                        volume: 1.0,
                        mute: false,
                        pan: 0.0,
                    },
                }],
                mixer: vec![MixerChannel {
                    effects: vec![
                        EffectInstance {
                            effect: Effect::SimpleEq {
                                config: EqConfig {
                                    kind: EqType::SimpleResonator,
                                    fc: 1000.0,
                                    q: 1.0,
                                    gain: 0.0,
                                },
                            },
                            meta: EffectMeta {
                                id: 0,
                                wet: 1.0,
                                mute: false,
                            },
                        },
                        EffectInstance {
                            effect: Effect::SimpleDelay {
                                config: DelayConfig {
                                    delay_ms: 250.0,
                                    feedback: 0.5,
                                },
                            },
                            meta: EffectMeta {
                                id: 1,
                                wet: 0.5,
                                mute: false,
                            },
                        },
                        EffectInstance {
                            effect: Effect::SimpleCompressor {
                                config: CompressorConfig {
                                    threshold: 0.3,
                                    attack_ms: 0.1,
                                    release_ms: 0.8,
                                    ratio: 1.5,
                                    gain: 1.0,
                                },
                            },
                            meta: EffectMeta {
                                id: 2,
                                wet: 1.0,
                                mute: false,
                            },
                        },
                        EffectInstance {
                            effect: Effect::ModDelay {
                                config: ModDelayConfig {
                                    min_depth: 100,
                                    max_depth: 200,
                                    freq: 10.0,
                                    lfo_type: WaveType::Triangle,
                                },
                            },
                            meta: EffectMeta {
                                id: 1,
                                wet: 0.5,
                                mute: false,
                            },
                        },
                    ],
                }],
                bpm: 120.0,
                mod_matrix: ModMatrix::default(),
            },
            volume: 1.0,
            key: ScaleValue::A,
            scale: Scale::Chromatic,
            sample_tree: None,
            sample_tree_config: FileTreeConfig {
                search: None,
                skip_non_audio: true,
                skip_hidden: true,
            },
            project_list: vec![],
            load_project_name: None,
        }
    }
}
