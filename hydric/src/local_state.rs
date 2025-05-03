use state::TrackSelector;
use std::cell::RefCell;
use std::rc::Rc;

type RcOption<T> = Rc<RefCell<Option<T>>>;

#[derive(Default)]
pub struct LocalState {
    pub active_track: RcOption<TrackSelector>,
    pub active_note: RcOption<usize>,
    pub mixer_edit_state: RcOption<bool>,
}

pub trait GetSetOption<T: Clone> {
    fn get(&self) -> Option<T>;

    fn set(&self, value: T);

    fn set_none(&self);
}

/// Implementation of borrowing a RefCell, reading or mutating, and then immediately finishing the borrow.
/// If this is the only interface through which a RefCell is used, it should be ~impossible to
/// cause panics.
impl<T: Clone> GetSetOption<T> for Rc<RefCell<Option<T>>> {
    fn get(&self) -> Option<T> {
        self.borrow().clone()
    }

    fn set(&self, value: T) {
        *self.borrow_mut() = Some(value);
    }

    fn set_none(&self) {
        *self.borrow_mut() = None;
    }
}
