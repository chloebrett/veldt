use super::{Action, reducer};
use ordered_float::OrderedFloat;
use shared::model::PlacedNote;
use shared::model::Track;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, Note, PitchName, Project, Scale,
    ScaleValue, SimpleWaveConfig, WaveType,
};
use shared::types::Volume;
use std::cell::{Ref, RefCell, RefMut};
use web_sys::console;

pub struct Store {
    data: RefCell<StoreData>,
}

pub struct StoreData {
    pub project: Project,
    pub key: ScaleValue,
    pub scale: Scale,
    pub volume: Volume,
}

impl Store {
    pub fn get(&self) -> Ref<'_, StoreData> {
        self.data.borrow()
    }

    /// Should be avoided where possible - use actions and pass around an immutable Store instead.
    // Note: self could just be an immutable reference, but that would be less safe.
    pub fn get_mut(&mut self) -> RefMut<'_, StoreData> {
        self.data.borrow_mut()
    }

    // Dispatching is allowed with only an immutable reference.
    // We mutate via the RefCell. This allows Store to be passed around mutably,
    // while allowing the caller to dispatch actions to it.
    // TODO: consider queueing actions for dispatch, which would make discarding frames from egui
    // unnecessary.
    pub fn dispatch(&self, action: Action) {
        console::log_1(&format!("Ran action: {:?}", action).into());
        reducer(self.data.borrow_mut(), action)
    }
}

impl Default for Store {
    fn default() -> Self {
        Store {
            data: RefCell::new(StoreData::default()),
        }
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
        }
    }
}
