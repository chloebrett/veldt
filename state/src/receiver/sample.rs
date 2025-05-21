use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::Sample;

impl ActionReceiver for Sample {
    fn apply(&mut self, _action: &Action) -> Option<Action> {
        None
    }
}
