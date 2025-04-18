use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::EffectMeta;

impl ActionReceiver for EffectMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetEffectWet(wet) => {
                let prev = self.wet;
                self.wet = *wet;
                Action::SetEffectWet(prev)
            }
            Action::SetEffectMute(mute) => {
                let prev = self.mute;
                self.mute = *mute;
                Action::SetEffectMute(prev)
            }
            _ => return None,
        })
    }
}
