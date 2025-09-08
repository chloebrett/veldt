use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::PlacedDrum;

impl ActionReceiver for PlacedDrum {
    fn apply(&mut self, _action: &Action) -> Option<Action> {
        // TODO implement receiver logic for all actions/types for a placed drum
        None
    }
}
