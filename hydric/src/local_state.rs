use state::TrackSelector;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct LocalState {
    pub active_track: Rc<RefCell<Option<TrackSelector>>>,
}

pub trait EasyBorrow<T: Clone> {
    fn borrow_get(&self) -> Option<T>;

    fn borrow_set(&self, value: T);

    fn borrow_set_none(&self);
}

impl<T: Clone> EasyBorrow<T> for Rc<RefCell<Option<T>>> {
    fn borrow_get(&self) -> Option<T> {
        let cell: Rc<RefCell<Option<T>>> = (*self).clone();
        cell.borrow().clone()
    }

    fn borrow_set(&self, value: T) {
        let cell: Rc<RefCell<Option<T>>> = (*self).clone();
        *cell.borrow_mut() = Some(value);
    }

    fn borrow_set_none(&self) {
        let cell: Rc<RefCell<Option<T>>> = (*self).clone();
        *cell.borrow_mut() = None;
    }
}
