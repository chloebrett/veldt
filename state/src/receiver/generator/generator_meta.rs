use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::GeneratorMeta;

impl ActionReceiver for GeneratorMeta {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetVolume(volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetVolume(prev)
            }
            Action::SetMute(mute) => {
                let prev = self.mute;
                self.mute = *mute;
                Action::SetMute(prev)
            }
            Action::SetPan(pan) => {
                let prev = self.pan;
                self.pan = *pan;
                Action::SetPan(prev)
            }
            _ => return None,
        })
    }
}
