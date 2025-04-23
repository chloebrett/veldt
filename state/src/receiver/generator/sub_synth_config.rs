use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::SubSynthConfig;

impl ActionReceiver for SubSynthConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        for osc in &mut self.oscillators {
            if let Some(result) = osc.apply(action) {
                return Some(result);
            }
        }
        None
    }
}
