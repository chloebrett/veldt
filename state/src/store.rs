use crate::{MultiIndexField, TrackSelector};
use crate::{
    Action, ReversibleAction, RootSelector, Selector, SelectorTrait, StoreData, UndoStack,
    receiver::ActionReceiver,
};
use log::info;
use shared::model::TrackId;
use std::cell::RefCell;
use std::sync::mpsc::Sender;

enum UndoRedoType {
    Undo,
    Redo,
}

pub struct Store {
    // The canonical view of the store state, which can be mutated indirectly through actions.
    data: StoreData,

    // The actions that have been queued to run but not run yet.
    // Each frame, the pending actions are applied to the state in order.
    // Contained within a RefCell so that it can be mutated with only an immutable reference to the
    // store. This is safe because it's only borrowed for the duration of the dispatch() function,
    // and it isn't read until we flush the actions at the start of the
    // next frame.
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
    pub fn new(
        broadcast: impl Fn(Vec<ReversibleAction>) + Send + 'static,
        tx: Sender<(Selector, Action)>,
    ) -> Self {
        Store {
            data: StoreData::default(),
            pending_actions: RefCell::new(vec![]),
            undo_stack: UndoStack::new(broadcast, tx),
            pending_undo_redo: None,
        }
    }

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
            info!("Applying action: {:?}", action.clone());
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

    pub fn try_select<'a, T: ActionReceiver + 'a, S: SelectorTrait<Item = T> + 'a>(
        &'a self,
        selector: &'a S,
    ) -> Option<&'a T> {
        selector.try_select(self.get())
    }

    pub fn select<'a, T: ActionReceiver + 'a, S: SelectorTrait<Item = T> + 'a>(
        &'a self,
        selector: &'a S,
    ) -> &'a T {
        selector.select(self.get())
    }

    pub fn cleanup_deleted_notes(&self, track_id: TrackId) {
        let notes = self.select(&TrackSelector(track_id)).notes.clone();
        let mut to_delete = Vec::new();

        for (i, note) in notes.iter().enumerate() {
            if note.note_deleted {
                to_delete.push(i);
            }
        }

        if !to_delete.is_empty() {
            self.dispatch(
                &TrackSelector(track_id),
                Action::DeleteChildren(MultiIndexField::PlacedNote(to_delete)),
            );
        }
    }

    // Dispatching is allowed with only an immutable reference.
    // We mutate via the RefCell containing the queued actions. This allows Store to be passed around immutably,
    // while allowing the caller to dispatch actions to it. As long as dispatch() is only called in
    // a single thread, which is the case in WASM, this is safe. If it needs to be sent across
    // threads, it should be replaced with a Mutex.
    // TODO: migrate all `dispatch` usages to this, then remove the old dispatch method.
    pub fn dispatch<'a, T: ActionReceiver + 'a, S: SelectorTrait<Item = T> + 'a>(
        &self,
        selector: &S,
        action: Action,
    ) {
        info!("Recording action: {:?}", action.clone());
        self.pending_actions
            .borrow_mut()
            // TODO: use the selector trait deeper in the store?
            // Consider this once we've stopped using the old `dispatch`.
            .push((selector.as_enum().clone(), action.clone()));
    }

    // Dispatches an action with an enum selector.
    // Prefer the trait-based dispatch method where possible.
    pub fn dispatch_enum(&self, selector: &Selector, action: Action) {
        info!("Recording action: {:?}", action.clone());
        self.pending_actions
            .borrow_mut()
            .push((selector.clone(), action.clone()));
    }

    /// Shorthand for dispatch(Selector::Root, ..)
    pub fn dispatchr(&self, action: Action) {
        self.dispatch(&RootSelector, action)
    }
}
