use state::TrackSelector;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct LocalState {
    pub active_track: Rc<RefCell<Option<TrackSelector>>>,
}
