use super::ActionReceiver;
use crate::Action;
use shared::model::CompressorConfig;

impl ActionReceiver for CompressorConfig {
    fn apply(&mut self, action: &Action) -> Action {
        match action {
            Action::SetCompressorThreshold(volume) => {
                let prev = self.threshold;
                self.threshold = *volume;
                Action::SetCompressorThreshold(prev)
            }
            Action::SetCompressorAttackMs(attack_ms) => {
                let prev = self.attack_ms;
                self.attack_ms = *attack_ms;
                Action::SetCompressorAttackMs(prev)
            }
            Action::SetCompressorReleaseMs(release_ms) => {
                let prev = self.release_ms;
                self.release_ms = *release_ms;
                Action::SetCompressorReleaseMs(prev)
            }
            Action::SetCompressorRatio(ratio) => {
                let prev = self.ratio;
                self.ratio = *ratio;
                Action::SetCompressorRatio(prev)
            }
            Action::SetCompressorGain(gain) => {
                let prev = self.gain;
                self.gain = *gain;
                Action::SetCompressorGain(prev)
            }
            _ => Action::NonReversible,
        }
    }
}
