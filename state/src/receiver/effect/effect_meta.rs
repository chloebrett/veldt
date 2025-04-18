use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::EffectMeta;

impl ActionReceiver for EffectMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::Wet, wet) => {
                let prev = self.wet;
                self.wet = *wet;
                Action::SetFloat(FloatField::Wet, prev)
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
