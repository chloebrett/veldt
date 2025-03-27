use super::{Action, Selector, StoreData, root_reducer};
use shared::logger::log;

/// An action that can be applied forwards or backwards.
#[derive(Clone, Debug)]
struct ReversibleAction {
    pub selector: Selector,
    pub forward: Action,
    pub reverse: Action,
}

#[derive(Clone, Default, Debug)]
pub struct UndoStack {
    // Actions that have been applied at least once. Includes actions that have been undone.
    // If a new action is applied while there are undone actions, the undone actions are discarded.
    actions: Vec<ReversibleAction>,

    // The index of the current position in applied_actions. Increases by one every time an action
    // is performed.
    index: usize,
}

impl UndoStack {
    /// Performs an action for the first time.
    /// Saves instructions for how to undo the action so that it can be undone in future.
    pub fn apply(&mut self, store: &mut StoreData, selector: &Selector, action: &Action) {
        // Run the action, and remember how to reverse it.
        let reverse = root_reducer(store, selector, action);

        // Discard any available redos in the stack.
        if self.index < self.actions.len() {
            self.actions.truncate(self.index);
        }

        self.actions.push(ReversibleAction {
            selector: selector.clone(),
            forward: action.clone(),
            reverse,
        });
        self.index += 1;
    }

    /// Whether the stack has actions that can be undone.
    pub fn can_undo(&self) -> bool {
        self.index > 0
    }

    /// Undoes the action at the top of the stack.
    pub fn undo(&mut self, store: &mut StoreData) {
        if !self.can_undo() {
            panic!("Tried to undo when there was nothing to undo!");
        }

        log(&format!(
            "Undoing action. Stack state before: {:?}",
            self.clone()
        ));
        let action = self
            .actions
            .get(self.index - 1)
            .expect("UndoStack index was invalid!");
        root_reducer(store, &action.selector, &action.reverse);
        self.index -= 1;
    }

    /// Whether the stack has actions that can be redone.
    pub fn can_redo(&self) -> bool {
        self.index < self.actions.len()
    }

    /// Redoes the action at the current index in the stack.
    pub fn redo(&mut self, store: &mut StoreData) {
        if !self.can_redo() {
            panic!("Tried to redo when there was nothing to redo!");
        }

        log(&format!(
            "Redoing action. Stack state before: {:?}",
            self.clone()
        ));
        let action = self
            .actions
            .get(self.index)
            .expect("UndoStack index was invalid!");
        root_reducer(store, &action.selector, &action.forward);
        self.index += 1;
    }
}
