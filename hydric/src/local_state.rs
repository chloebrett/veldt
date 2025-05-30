use egui::Pos2;
use shared::model::Sample;
use state::{MixerSelector, TrackSelector};
use std::cell::RefCell;
use std::{borrow as std_borrow};
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

type RcOption<T> = Rc<RefCell<Option<T>>>;

#[derive(Default)]
pub struct LocalState {
    pub active_track: RcOption<TrackSelector>,
    pub active_note: RcOption<usize>,
    pub active_placement: RcOption<usize>,
    pub active_mixer_channel: RcOption<MixerSelector>,

    pub selected_notes: Rc<RefCell<HashSet<usize>>>,
    pub selected_placements: Rc<RefCell<HashSet<usize>>>,

    pub mixer_edit_state: Rc<RefCell<bool>>,

    pub note_window: Rc<RefCell<bool>>,
    pub note_roll_window: Rc<RefCell<bool>>,
    pub placement_window: Rc<RefCell<bool>>,

    pub drag_cursor_delta: RcOption<Pos2>,

    // TODO: in the case of multiple stingray instances, both env and lfo should be in a HashSet
    // with the generator id as the key
    pub stingray_env_tab: Rc<RefCell<usize>>,
    pub stingray_lfo_tab: Rc<RefCell<usize>>,

    pub track_roll_select_enabled: Rc<RefCell<bool>>,
    pub note_roll_select_enabled: Rc<RefCell<bool>>,

    pub sample_cache: Rc<RefCell<HashMap<String, Sample>>>,
}

pub trait GetSet<T: Clone> {
    fn get(&self) -> T;

    fn set(&self, value: T);

    fn update(&self, closure: impl Fn(T) -> T);
}

/// Implementation of borrowing a RefCell, reading or mutating, and then immediately finishing the borrow.
/// If this is the only interface through which a RefCell is used, it should be ~impossible to
/// cause panics.
impl<T: Clone> GetSet<T> for Rc<RefCell<T>> {
    fn get(&self) -> T {
        self.borrow().clone()
    }

    fn set(&self, value: T) {
        *self.borrow_mut() = value;
    }

    fn update(&self, closure: impl Fn(T) -> T) {
        let old = self.get();
        self.set(closure(old));
    }
}

pub trait HashMapOperations<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn hashmap_insert(&self, key: K, value: V) -> Option<V>;

    fn hashmap_get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: std_borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash;

    // Currently not used anywhere so it's commented out to avoid lints but could be useful
    // fn hashmap_update<Q: ?Sized, F>(&self, key: &Q, f: F) -> bool
    // where
    //     K: std_borrow::Borrow<Q>,
    //     Q: Eq + std::hash::Hash,
    //     F: FnOnce(&mut V);

    fn hashmap_contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: std_borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash;
}


impl<K, V> HashMapOperations<K, V> for Rc<RefCell<HashMap<K, V>>>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn hashmap_insert(&self, key: K, value: V) -> Option<V> {
        self.borrow_mut().insert(key, value)
    }

    fn hashmap_get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: std_borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        self.borrow().get(key).cloned()
    }

    // fn hashmap_update<Q: ?Sized, F>(&self, key: &Q, f: F) -> bool
    // where
    //     K: std_borrow::Borrow<Q>,
    //     Q: Eq + std::hash::Hash,
    //     F: FnOnce(&mut V),
    // {
    //     if let Some(value) = self.borrow_mut().get_mut(key) {
    //         f(value);
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn hashmap_contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: std_borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        self.borrow().contains_key(key)
    }
}