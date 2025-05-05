use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, TypeField};
use shared::model::GeneratorMeta;

impl ActionReceiver for GeneratorMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetChild(TypeField::Mute(mute)) => {
                let prev = self.mute;
                self.mute = *mute;
                Action::SetChild(TypeField::Mute(prev))
            }
            Action::SetFloat(FloatField::Pan, pan) => {
                let prev = self.pan;
                self.pan = *pan;
                Action::SetFloat(FloatField::Pan, prev)
            }
            Action::SetIndex(IndexField::Mixer(mixer_channel)) => {
                let prev = self.mixer_channel;
                self.mixer_channel = *mixer_channel;
                Action::SetIndex(IndexField::Mixer(prev))
            }
            _ => return None,
        })
    }
}
