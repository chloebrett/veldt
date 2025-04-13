use crate::{Action, BroadcastType, Selector, StoreData, broadcast_type, root_reducer};
use local_macro::{FromProto, IntoProto};
use shared::action_proto::ReversibleActionProto;
use shared::logger::log;
use std::mem::discriminant;

/// An action that can be applied forwards or backwards.
#[derive(Clone, Debug, IntoProto, FromProto)]
pub struct ReversibleAction {
    #[proto_optional]
    pub selector: Selector,

    #[proto_optional]
    pub forward: Action,

    #[proto_optional]
    pub reverse: Action,

    pub is_compacted: bool,
}

pub struct UndoStack {
    // Broadcasts a set of actions (to the server).
    // TODO: should probably be an async type.
    broadcast: Box<dyn Fn(Vec<ReversibleAction>) + Send>,

    // Actions that have been applied at least once. Includes actions that have been undone.
    // If a new action is applied while there are undone actions, the undone actions are discarded.
    actions: Vec<ReversibleAction>,

    // The index of the current position in applied_actions. Increases by one every time an action
    // is performed. If there is nothing to redo, this is equal to actions.len().
    index: usize,
}

impl UndoStack {
    pub fn new(broadcast: impl Fn(Vec<ReversibleAction>) + Send + 'static) -> Self {
        UndoStack {
            broadcast: Box::new(broadcast),
            actions: vec![],
            index: 0,
        }
    }

    /// Performs an action for the first time.
    /// Saves instructions for how to undo the action so that it can be undone in future.
    pub fn apply(&mut self, store: &mut StoreData, selector: &Selector, action: &Action) {
        // Special case: "release" marker actions should flatten actions of the same type that came before them.
        if *action == Action::Release {
            log("Got release action");
            self.compact_last_actions();
            return;
        }

        // Run the action, and remember how to reverse it.
        let reverse = root_reducer(store, selector, action);

        // Discard any available redos in the stack.
        if self.index < self.actions.len() {
            self.actions.truncate(self.index);
        }

        let reversible_action = ReversibleAction {
            selector: selector.clone(),
            forward: action.clone(),
            reverse,
            is_compacted: false,
        };
        self.actions.push(reversible_action.clone());

        self.index += 1;

        // TODO: store broadcast state of past actions / index of how far we have broadcasted.
        if broadcast_type(action) == BroadcastType::Immediate {
            self.broadcast(reversible_action.clone());
        }
    }

    /// Compacts the last actions of the same type into a single action.
    /// This enables the undo/redo stack to handle floats appropriately.
    fn compact_last_actions(&mut self) {
        // This should only happen if there is nothing to redo, and at least one stored action.
        debug_assert!(self.actions.len() > 0);
        debug_assert!(self.index == self.actions.len());
        log(&format!("Compacting actions! {:?}", self.actions,));

        let last_action = &self.actions.last().unwrap();
        let last_action_type = discriminant(&last_action.forward);
        let last_action_selector = self.actions.last().unwrap().selector.clone();

        let last_index = self.actions.len() - 1;
        let mut compact_from_index = last_index;
        for i in (0..last_index).rev() {
            let action = &self.actions[i];
            if discriminant(&action.forward) == last_action_type
                && action.selector == last_action_selector
                && !action.is_compacted
            {
                compact_from_index -= 1;
            } else {
                break;
            }
        }

        // The forward-action of the last action the user performed.
        let forward = self.actions[last_index].forward.clone();

        // Remove all actions that happened after the compact_from_index.
        self.actions.truncate(/* len= */ compact_from_index + 1);

        let new_last_action = &mut self.actions[compact_from_index];

        // Update the forward action to match what the user did, but leave the reverse.
        new_last_action.forward = forward;

        // Mark the action as compacted so that it doesn't get compacted again.
        new_last_action.is_compacted = true;

        // The index might now be lower than it was before.
        self.index = compact_from_index + 1;

        // Broadcast the action if appropriate.
        if broadcast_type(&new_last_action.forward) == BroadcastType::OnRelease {
            let action = new_last_action.clone();
            self.broadcast(action);
        }

        log(&format!(
            "Compacted actions! {:?}, {}",
            self.actions, compact_from_index,
        ));
    }

    fn broadcast(&self, action: ReversibleAction) {
        log(&format!("Broadcasting action to server! {:?}", action));
        (self.broadcast)(vec![action]);
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
            self.actions.clone()
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
            self.actions.clone()
        ));
        let action = self
            .actions
            .get(self.index)
            .expect("UndoStack index was invalid!");
        root_reducer(store, &action.selector, &action.forward);
        self.index += 1;
    }
}
