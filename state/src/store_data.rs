use crate::reducer;
use crate::{Action, Selector};
use ordered_float::OrderedFloat;
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, CompressorConfig, DelayConfig, Effect, EffectInstance,
    EffectMeta, EqConfig, EqType, FileTreeConfig, FilenameTree, GeneratorInstance, GeneratorMeta,
    GeneratorType, LfoConfig, MixerChannel, ModDelayConfig, ModMatrix, Note, OscillatorConfig,
    PitchName, PlacedNote, Project, Scale, ScaleValue, SimpleWaveConfig, SubSynthConfig, Track,
    TrackPlacement, WaveType,
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

impl StoreData {
    pub fn update(&mut self, selector: &Selector, action: &Action) -> Option<Action> {
        reducer(self, selector, action)
    }
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
                generators: vec![
                    GeneratorInstance {
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
                            config: SubSynthConfig {
                                oscillators: [
                                    OscillatorConfig {
                                        wave: WaveType::Sine,
                                        volume: 1.0,
                                        pan: 0.0,
                                        osc_detune: 0.0,
                                        osc_count: 1,
                                        unison_detune: 0.0,
                                    },
                                    OscillatorConfig {
                                        wave: WaveType::Triangle,
                                        volume: 1.0,
                                        pan: 0.0,
                                        osc_detune: 0.0,
                                        osc_count: 1,
                                        unison_detune: 0.0,
                                    },
                                    OscillatorConfig {
                                        wave: WaveType::Square,
                                        volume: 1.0,
                                        pan: 0.0,
                                        osc_detune: 0.0,
                                        osc_count: 1,
                                        unison_detune: 0.0,
                                    },
                                ],
                                lfos: [
                                    LfoConfig {
                                        wave: WaveType::Sine,
                                        frequency: 1.0,
                                    },
                                    LfoConfig {
                                        wave: WaveType::Sine,
                                        frequency: 1.0,
                                    },
                                    LfoConfig {
                                        wave: WaveType::Sine,
                                        frequency: 1.0,
                                    },
                                ],
                                envelopes: [
                                    AdsrEnvelope {
                                        attack: 0.1,
                                        decay: 0.1,
                                        sustain: 0.8,
                                        release: 0.1,
                                    },
                                    AdsrEnvelope {
                                        attack: 0.1,
                                        decay: 0.1,
                                        sustain: 0.8,
                                        release: 0.1,
                                    },
                                    AdsrEnvelope {
                                        attack: 0.1,
                                        decay: 0.1,
                                        sustain: 0.8,
                                        release: 0.1,
                                    },
                                ],
                            },
                        },
                        meta: GeneratorMeta {
                            volume: 1.0,
                            mute: false,
                            pan: 0.0,
                        },
                    },
                ],
                mixer: vec![MixerChannel { effects: vec![] }],
                bpm: 120.0,
                mod_matrix: ModMatrix::default(),
            },
            volume: 1.0,
            key: ScaleValue::A,
            scale: Scale::Chromatic,
            sample_tree: None,
            sample_tree_config: FileTreeConfig {
                search: "".to_string(),
                show_non_audio: false,
                show_hidden: false,
            },
            project_list: vec![],
            load_project_name: None,
        }
    }
}
