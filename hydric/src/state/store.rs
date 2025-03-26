use super::UndoStack;
use super::{Action, Selector};
use ordered_float::OrderedFloat;
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, MixerChannel, Note, PitchName, PlacedNote,
    Project, Sample, Scale, ScaleValue, SimpleWaveConfig, Track, WaveType,
};
use shared::types::Volume;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::console;

enum UndoRedoType {
    Undo,
    Redo,
}

#[derive(Default)]
pub struct Store {
    // The canonical view of the store state, which can be mutated indirectly through actions.
    data: StoreData,

    // The actions that have been queued to run but not run yet.
    // Each frame, the pending actions are applied to the state in order.
    // Contained within a RefCell so that it can be mutated with only an immutable reference to the
    // store.
    pending_actions: RefCell<Vec<(Selector, Action)>>,

    // State regarding actions that have been completed in the past and possibly undone.
    // Does not need to be in a refcell because it is only modified in snapshot().
    // TODO: consider making the reducers pure functions, as this might make commuting
    // actions and determining conflicts easier for collaborative editing. However, this seems
    // like it would require more cloning unless clever algorithms are used.
    undo_stack: UndoStack,

    /// The pending undo/redo, if one is pending. This will be flushed on the next frame,
    /// to avoid mutating the StoreData mid-frame.
    pending_undo_redo: Option<UndoRedoType>,
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
    pub load_sample_promise: Rc<Option<Promise<Option<Sample>>>>,
}

impl Store {
    /// Snapshots the state by applying all of the pending actions.
    pub fn snapshot(&mut self) {
        if let Some(undo_redo) = &self.pending_undo_redo {
            match undo_redo {
                UndoRedoType::Undo => self.undo_stack.undo(&mut self.data),
                UndoRedoType::Redo => self.undo_stack.redo(&mut self.data),
            }
            self.pending_undo_redo = None;
        }

        for (selector, action) in self.pending_actions.borrow().iter() {
            console::log_1(&format!("Applying action: {:?}", action.clone()).into());
            self.undo_stack.apply(&mut self.data, selector, action);
        }
        self.pending_actions.borrow_mut().clear();
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    pub fn pend_undo(&mut self) {
        self.pending_undo_redo = Some(UndoRedoType::Undo);
    }

    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    pub fn pend_redo(&mut self) {
        self.pending_undo_redo = Some(UndoRedoType::Redo);
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
