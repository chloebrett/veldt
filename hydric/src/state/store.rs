use super::{UndoStack, StoreData, Action, Selector};
use shared::logger::log;
use std::cell::RefCell;

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
            log(&format!("Applying action: {:?}", action.clone()));
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
        log(&format!("Recording action: {:?}", action.clone()));
        self.pending_actions
            .borrow_mut()
            .push((selector.clone(), action.clone()));
    }

    /// Shorthand for dispatch(Selector::Root, ..)
    pub fn dispatchr(&self, action: Action) {
        self.dispatch(&Selector::Root, action)
    }
}

