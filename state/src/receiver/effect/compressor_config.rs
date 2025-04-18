use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::CompressorConfig;

impl ActionReceiver for CompressorConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.threshold;
                self.threshold = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::SetFloat(FloatField::AttackMs, attack_ms) => {
                let prev = self.attack_ms;
                self.attack_ms = *attack_ms;
                Action::SetFloat(FloatField::AttackMs, prev)
            }
            Action::SetFloat(FloatField::ReleaseMs, release_ms) => {
                let prev = self.release_ms;
                self.release_ms = *release_ms;
                Action::SetFloat(FloatField::ReleaseMs, prev)
            }
            Action::SetFloat(FloatField::Ratio, ratio) => {
                let prev = self.ratio;
                self.ratio = *ratio;
                Action::SetFloat(FloatField::Ratio, prev)
            }
            Action::SetFloat(FloatField::Gain, gain) => {
                let prev = self.gain;
                self.gain = *gain;
                Action::SetFloat(FloatField::Gain, prev)
            }
            _ => return None,
        })
    }
}
