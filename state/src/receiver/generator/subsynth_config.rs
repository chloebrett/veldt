use crate::receiver::ActionReceiver;
use crate::Action;
use log::info;
use shared::model::SubSynthConfig;

impl ActionReceiver for SubSynthConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        info!("subsynth happened");
        Some(match action {
            // TODO add actions
            _ => return None,
        })
    }
}
