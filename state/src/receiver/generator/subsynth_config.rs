use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField, UintField};
use shared::model::SubSynthConfig;
use log::info;

impl ActionReceiver for SubSynthConfig {
    fn apply(&mut self, action: &Action ) -> Option<Action> {
        info!("subsynth happened");
        Some(match action {
            // TODO add actions
            _ => return None,
        })
    }
}
