use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::EqConfig;

impl ActionReceiver for EqConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetEqKind(kind) => {
                let prev = self.kind.clone();
                self.kind = kind.clone();
                Action::SetEqKind(prev)
            }
            Action::SetEqFc(fc) => {
                let prev = self.fc;
                self.fc = *fc;
                Action::SetEqFc(prev)
            }
            Action::SetEqQ(q) => {
                let prev = self.q;
                self.q = *q;
                Action::SetEqQ(prev)
            }
            Action::SetGain(gain) => {
                let prev = self.gain;
                self.gain = *gain;
                Action::SetGain(prev)
            }
            _ => return None,
        })
    }
}
