use state::TrackSelector;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct LocalState {
    pub active_track: Rc<RefCell<Option<TrackSelector>>>,
}

pub fn borrow_get<T: Clone>(field: &Rc<RefCell<T>>) -> T {
    let cell: Rc<RefCell<T>> = (*field).clone();
    cell.borrow().clone()
}

pub fn borrow_set<T>(field: &Rc<RefCell<Option<T>>>, value: T) {
    let cell: Rc<RefCell<Option<T>>> = (*field).clone();
    *cell.borrow_mut() = Some(value);
}

pub fn _borrow_set_none<T>(field: &Rc<RefCell<Option<T>>>) {
    let cell: Rc<RefCell<Option<T>>> = (*field).clone();
    *cell.borrow_mut() = None;
}
