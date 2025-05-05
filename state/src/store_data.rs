use crate::receiver::ActionReceiver;
use crate::{Action, Selector, SelectorTrait, reducer};
use ordered_float::OrderedFloat;
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, FileTreeConfig, FilenameTree, Generator, GeneratorInstance,
    GeneratorMeta, LfoConfig, Mixer, MixerChannel, ModMatrix, Note, Oscillator, PitchName,
    PlacedNote, Placement, PlacementType, Project, Scale, ScaleValue, SimpleWaveConfig,
    SubSynthConfig, Track, TrackPlacement, WaveType,
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

    pub fn select<'a, T: ActionReceiver + 'a, S: SelectorTrait<Item = T> + 'a>(
        &'a self,
        selector: &'a S,
    ) -> &'a T {
        selector.select(self)
    }
}

const BASE_OSC: Oscillator = Oscillator {
    wave: WaveType::Sine,
    volume: 1.0,
    pan: 0.0,
    osc_detune: 0.0,
    osc_count: 1,
    unison_detune: 0.0,
};

const BASE_LFO: LfoConfig = LfoConfig {
    wave: WaveType::Sine,
    frequency: 1.0,
};

const BASE_ENV: AdsrEnvelope = AdsrEnvelope {
    attack: 0.1,
    decay: 0.1,
    sustain: 0.8,
    release: 0.1,
};

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
                placements: vec![Placement {
                    kind: PlacementType::Track(TrackPlacement {
                        track_index: 0,
                        generator_index: 0,
                    }),
                    offset: 0.0.into(),
                    clipped_duration: None,
                    visual_placement: 0,
                }],
                samples: vec![],
                generators: vec![
                    GeneratorInstance {
                        it: Generator::SimpleWave(SimpleWaveConfig {
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
                        }),
                        meta: GeneratorMeta {
                            volume: 1.0,
                            mute: false,
                            pan: 0.0,
                            mixer_channel: 2,
                        },
                    },
                    GeneratorInstance {
                        it: Generator::SubSynth(SubSynthConfig {
                            oscillators: [
                                Oscillator {
                                    wave: WaveType::Sine,
                                    ..BASE_OSC
                                },
                                Oscillator {
                                    wave: WaveType::Triangle,
                                    ..BASE_OSC
                                },
                                Oscillator {
                                    wave: WaveType::Square,
                                    ..BASE_OSC
                                },
                            ],
                            lfos: [BASE_LFO; 3],
                            envelopes: [BASE_ENV; 3],
                            matrix: ModMatrix::new(6, 3),
                        }),
                        meta: GeneratorMeta {
                            volume: 1.0,
                            mute: false,
                            pan: 0.0,
                            mixer_channel: 2,
                        },
                    },
                ],
                mixer: Mixer {
                    channels: vec![
                        MixerChannel {
                            volume: 1.0,
                            effects: vec![],
                        },
                        MixerChannel {
                            volume: 1.0,
                            effects: vec![],
                        },
                        MixerChannel {
                            volume: 1.0,
                            effects: vec![],
                        },
                    ],
                },
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
