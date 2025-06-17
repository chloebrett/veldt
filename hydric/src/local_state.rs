use crate::WindowState;
use egui::Pos2;
use shared::model::{PlacementId, Sample};
use state::{MixerSelector, TrackSelector};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

type RcOption<T> = Rc<RefCell<Option<T>>>;

#[derive(Default)]
pub struct LocalState {
    pub active_track: RcOption<TrackSelector>,
    pub active_note: RcOption<usize>,
    pub active_placement: RcOption<PlacementId>,
    pub active_mixer_channel: RcOption<MixerSelector>,

    pub selected_notes: Rc<RefCell<HashSet<usize>>>,
    pub selected_placements: Rc<RefCell<HashSet<PlacementId>>>,

    pub mixer_edit_state: Rc<RefCell<bool>>,

    pub drag_cursor_delta: RcOption<Pos2>,

    // TODO: in the case of multiple stingray instances, both env and lfo should be in a HashSet
    // with the generator id as the key
    pub stingray_env_tab: Rc<RefCell<usize>>,
    pub stingray_lfo_tab: Rc<RefCell<usize>>,

    pub track_roll_select_enabled: Rc<RefCell<bool>>,

    pub window_state: WindowState,

    pub sample_cache: Rc<RefCell<HashMap<String, Sample>>>,

    pub note_roll_zoom: Rc<RefCell<f32>>,
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
