use super::{Action, Selector, root_reducer};
use ordered_float::OrderedFloat;
use poll_promise::Promise;
use shared::model::PlacedNote;
use shared::model::Track;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, Note, PitchName, Project, Scale,
    ScaleValue, SimpleWaveConfig, WaveType,
};
use shared::types::Volume;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::console;

pub struct Store {
    // The canonical view of the store state, which can be mutated indirectly through actions.
    data: StoreData,

    // The actions that have been queued to run but not run yet.
    // Each frame, the pending actions are applied to the state in order.
    // Contained within a RefCell so that it can be mutated with only an immutable reference to the
    // store.
    pending_actions: RefCell<Vec<(Selector, Action)>>,
}

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
}

impl Store {
    /// Snapshots the state by applying all of the pending actions.
    pub fn snapshot(&mut self) {
        for (selector, action) in self.pending_actions.borrow().iter() {
            console::log_1(&format!("Applying action: {:?}", action.clone()).into());
            root_reducer(&mut self.data, &selector, &action);
        }
        self.pending_actions.borrow_mut().clear();
    }

    pub fn get(&self) -> &StoreData {
        &self.data
    }

    // Dispatching is allowed with only an immutable reference.
    // We mutate via the RefCell containing the queued actions. This allows Store to be passed around immutably,
    // while allowing the caller to dispatch actions to it. As long as dispatch() is only called in
    // a single thread, which is the case in WASM, this is safe. If it needs to be sent across
    // threads, it should be replaced with a Mutex.
    pub fn dispatch(&self, selector: &Selector, action: Action) {
        console::log_1(&format!("Recording action: {:?}", action.clone()).into());
        self.pending_actions
            .borrow_mut()
            .push((selector.clone(), action.clone()));
    }

    /// Shorthand for dispatch(Selector::Root, ..)
    pub fn dispatchr(&self, action: Action) {
        self.dispatch(&Selector::Root, action)
    }
}

impl Default for Store {
    fn default() -> Self {
        Store {
            data: StoreData::default(),
            // Default capacity is 10 because more actions than this per frame is unlikely.
            pending_actions: Vec::with_capacity(10).into(),
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
            track_list: vec![],
            save_track_promise: Rc::new(None),
            track_list_promise: Rc::new(None),
            load_track_promise: Rc::new(None),
            load_track_name: None,
        }
    }
}
