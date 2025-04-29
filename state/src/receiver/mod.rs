use crate::Action;

mod effect;
mod generator;
mod mixer_channel;
mod note;
mod project;
mod store_data;
mod track;
mod placement;

/// A model object that can receive actions.
pub trait ActionReceiver {
    /// Tries to apply an action to a model object.
    /// If the action is applied, returns Some(action) where `action` reverses the change.
    /// If the action isn't relevant, returns None.
    fn apply(&mut self, action: &Action) -> Option<Action>;
}
