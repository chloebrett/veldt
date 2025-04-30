use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::{Effect, EffectInstance};

impl ActionReceiver for EffectInstance {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        if let Some(undo) = self.meta.apply(action) {
            return Some(undo);
        }

        match &mut self.it {
            Effect::Delay(config) => config.apply(action),
            Effect::SimpleEq(config) => config.apply(action),
            Effect::Compressor(config) => config.apply(action),
            Effect::ModDelay(config) => config.apply(action),
        }
    }
}
