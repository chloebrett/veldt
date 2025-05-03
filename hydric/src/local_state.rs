use state::TrackSelector;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

type RcOption<T> = Rc<RefCell<Option<T>>>;

#[derive(Default)]
pub struct LocalState {
    pub active_track: RcOption<TrackSelector>,
    pub active_note: RcOption<usize>,
    pub selected_notes: RcOption<BTreeSet<usize>>,
    pub mixer_edit_state: Rc<RefCell<bool>>,
    pub note_window: Rc<RefCell<bool>>,
    pub note_roll_window: Rc<RefCell<bool>>,
}

pub trait GetSet<T: Clone> {
    fn get(&self) -> T;

    fn set(&self, value: T);
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
}
