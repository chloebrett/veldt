use crate::receiver::ActionReceiver;
use crate::{Action, Selector, SelectorTrait, reducer};
use ordered_float::OrderedFloat;
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, FileTreeConfig, FilenameTree, Generator, GeneratorInstance,
    GeneratorMeta, Mixer, MixerChannel, MixerMatrix, NoiseConfig, Note, PitchName, PlacedNote,
    Placement, PlacementType, PolyphonyMode, Project, Scale, ScaleValue, SimpleWaveConfig,
    StingrayConfig, Track, TrackPlacement, WaveType, GeneratorId
};
use shared::types::Volume;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StoreData {
    pub project: Project,
    // TODO: stop using a main volume here.
    // Instead use the mixer 0 (main mixer channel) volume to control the overall volume.
    // Need to route samples through the mixer 0 output amp as well then.
    // Then we don't have to send any more of the StoreData (the rest isn't rendering-related) to
    // mesic.
    pub volume: Volume,

    // Non-rendering-related below:
    pub key: ScaleValue,
    pub scale: Scale,
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

    pub fn try_select<'a, T: ActionReceiver + 'a, S: SelectorTrait<Item = T> + 'a>(
        &'a self,
        selector: &'a S,
    ) -> Option<&'a T> {
        selector.try_select(self)
    }
}

impl Default for StoreData {
    fn default() -> Self {
        const EMPTY_CHANNEL: MixerChannel = MixerChannel {
            volume: 1.0,
            effects: vec![],
        };

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
                generators: HashMap::from([
                    (
                        GeneratorId(0),
                        GeneratorInstance {
                            it: Generator::SimpleWave(SimpleWaveConfig {
                                wave: WaveType::Sine,
                                envelope: AdsrEnvelope {
                                    attack: 100.0,
                                    decay: 100.0,
                                    sustain: 0.8,
                                    release: 100.0,
                                },
                                osc_count: 4,
                                detune_cents: 5.0,
                                anti_aliasing_mode: AntiAliasingMode::Off,
                                oversample_factor: 2,
                                polyphony_mode: PolyphonyMode::Polyphonic,
                                polyphony_limit: 2,
                            }),
                            meta: GeneratorMeta {
                                volume: 1.0,
                                mute: false,
                                pan: 0.0,
                                mixer_channel: 2,
                            },
                        },
                    ),
                    (
                        GeneratorId(1),
                        GeneratorInstance {
                            it: Generator::Stingray(StingrayConfig::default()),
                            meta: GeneratorMeta {
                                volume: 1.0,
                                mute: false,
                                pan: 0.0,
                                mixer_channel: 2,
                            },
                        },
                    ),
                    (
                        GeneratorId(2),
                        GeneratorInstance {
                            it: Generator::Noise(NoiseConfig::default()),
                            meta: GeneratorMeta {
                                volume: 1.0,
                                mute: false,
                                pan: 0.0,
                                mixer_channel: 2,
                            },
                        },
                    ),
                    ]),
                mixer: Mixer {
                    matrix: MixerMatrix::with_channels(3),
                    channels: vec![EMPTY_CHANNEL; 3],
                },
                bpm: 120.0,
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
