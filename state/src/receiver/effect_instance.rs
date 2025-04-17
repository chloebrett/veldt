use super::ActionReceiver;
use crate::Action;
use shared::model::{Effect, EffectInstance};

impl ActionReceiver for EffectInstance {
    fn apply(&mut self, action: &Action) -> Action {
        match action {
            Action::SetEffectWet(wet) => {
                let prev = self.meta.wet;
                self.meta.wet = *wet;
                return Action::SetEffectWet(prev);
            }
            Action::SetEffectMute(mute) => {
                let prev = self.meta.mute;
                self.meta.mute = *mute;
                return Action::SetEffectMute(prev);
            }
            _ => {}
        }

        match &mut self.effect {
            Effect::SimpleDelay { config } => config.apply(action),
            Effect::SimpleEq { config } => config.apply(action),
            Effect::SimpleCompressor { config } => config.apply(action),
            Effect::ModDelay { config } => config.apply(action),
        }
    }
}
