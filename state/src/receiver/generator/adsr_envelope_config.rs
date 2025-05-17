use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::AdsrEnvelope;

impl ActionReceiver for AdsrEnvelope {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::AdsrAttack, attack) => {
                let prev = self.attack;
                self.attack = *attack;
                Action::SetFloat(FloatField::AdsrAttack, prev)
            }
            Action::SetFloat(FloatField::AdsrDecay, decay) => {
                let prev = self.decay;
                self.decay = *decay;
                Action::SetFloat(FloatField::AdsrDecay, prev)
            }
            Action::SetFloat(FloatField::AdsrSustain, sustain) => {
                let prev = self.sustain;
                self.sustain = *sustain;
                Action::SetFloat(FloatField::AdsrSustain, prev)
            }
            Action::SetFloat(FloatField::AdsrRelease, release) => {
                let prev = self.release;
                self.release = *release;
                Action::SetFloat(FloatField::AdsrRelease, prev)
            }
            _ => return None,
        })
    }
}
