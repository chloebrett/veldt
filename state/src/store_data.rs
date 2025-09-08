use crate::receiver::ActionReceiver;
use crate::{Action, Selector, SelectorTrait, reducer};
use ordered_float::OrderedFloat;
use shared::model::{
    AdsrEnvelope, AntiAliasingMode, Colour, FileTreeConfig, FilenameTree, Generator, GeneratorId,
    GeneratorInstance, GeneratorMeta, Mixer, MixerChannel, MixerMatrix, NoiseConfig, Note,
    PitchName, PlacedNote, Placement, PlacementId, PlacementType, PolyphonyMode, Project, Scale,
    ScaleValue, SimpleWaveConfig, StingrayConfig, Track, TrackId, TrackPlacement, WaveType,
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
        StoreData {
            project: Project {
                name: "My Project".to_string(),
                tracks: HashMap::from([(
                    TrackId(0),
                    Track {
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
                    },
                )]),
                placements: HashMap::from([(
                    PlacementId(0),
                    Placement {
                        kind: PlacementType::Track(TrackPlacement {
                            track_id: 0.into(),
                            generator_id: 0.into(),
                        }),
                        offset: 0.0.into(),
                        clipped_duration: None,
                        visual_placement: 0,
                        colour: Colour::from_8bit(67, 206, 222),
                    },
                )]),
                samples: HashMap::new(),
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
                                name: "".to_string(),
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
                                name: "".to_string(),
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
                                name: "".to_string(),
                            },
                        },
                    ),
                ]),
                mixer: Mixer {
                    matrix: MixerMatrix::with_channels(3),
                    channels: vec![MixerChannel::default(); 3],
                },
                effects: HashMap::new(),
                bpm: 120.0,
                drum_tracks: HashMap::new(),
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
