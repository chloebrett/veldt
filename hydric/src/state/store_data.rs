use ordered_float::OrderedFloat;
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, Note, PitchName, PlacedNote,
    Project, Sample, Scale, ScaleValue, SimpleWaveConfig, Track, WaveType,
};
use shared::types::Volume;
use std::rc::Rc;

#[derive(Clone)]
pub struct StoreData {
    pub project: Project,
    pub key: ScaleValue,
    pub scale: Scale,
    pub volume: Volume,
    // TODO: call this "project_list"?
    pub track_list: Vec<String>,
    pub load_track_name: Option<String>,
    pub save_track_promise: Rc<Option<Promise<Option<()>>>>,
    pub track_list_promise: Rc<Option<Promise<Option<Vec<String>>>>>,
    pub load_track_promise: Rc<Option<Promise<Option<Track>>>>,
    pub load_sample_promise: Rc<Option<Promise<Option<Sample>>>>,
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
                }],
                // TODO: use track placements
                track_placements: vec![],
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
                                config: DelayConfig {
                                    amplitude: 0.5,
                                    delay_ms: 250.0,
                                },
                            },
                            meta: EffectMeta { id: 1, wet: 0.5 },
                        },
                    ],
                }],
                bpm: 120.0,
            },
            volume: 1.0,
            key: ScaleValue::A,
            scale: Scale::Chromatic,
            track_list: vec![],
            save_track_promise: Rc::new(None),
            track_list_promise: Rc::new(None),
            load_track_promise: Rc::new(None),
            load_sample_promise: Rc::new(None),
            load_track_name: None,
        }
    }
}
