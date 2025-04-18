use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, TypeField};
use shared::model::EffectMeta;

impl ActionReceiver for EffectMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::Wet, wet) => {
                let prev = self.wet;
                self.wet = *wet;
                Action::SetFloat(FloatField::Wet, prev)
            }
            Action::SetChild(TypeField::Mute(mute)) => {
                let prev = self.mute;
                self.mute = *mute;
                Action::SetChild(TypeField::Mute(prev))
            }
            _ => return None,
        })
    }
}
