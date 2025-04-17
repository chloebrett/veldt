use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::GeneratorMeta;

impl ActionReceiver for GeneratorMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetGeneratorVolume(volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetGeneratorVolume(prev)
            }
            Action::SetGeneratorMute(mute) => {
                let prev = self.mute;
                self.mute = *mute;
                Action::SetGeneratorMute(prev)
            }
            Action::SetGeneratorPan(pan) => {
                let prev = self.pan;
                self.pan = *pan;
                Action::SetGeneratorPan(prev)
            }
            _ => return None,
        })
    }
}
