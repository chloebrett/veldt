use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::EffectMeta;

impl ActionReceiver for EffectMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetWet(wet) => {
                let prev = self.wet;
                self.wet = *wet;
                Action::SetWet(prev)
            }
            Action::SetMute(mute) => {
                let prev = self.mute;
                self.mute = *mute;
                Action::SetMute(prev)
            }
            _ => return None,
        })
    }
}
