use super::ActionReceiver;
use crate::Action;
use shared::model::DelayConfig;

impl ActionReceiver for DelayConfig {
    fn apply(&mut self, action: &Action) -> Action {
        match action {
            Action::SetDelayMs(delay_ms) => {
                let prev = self.delay_ms;
                self.delay_ms = *delay_ms;
                Action::SetDelayMs(prev)
            }
            Action::SetDelayFeedback(feedback) => {
                let prev = self.feedback;
                self.feedback = *feedback;
                Action::SetDelayFeedback(prev)
            }
            _ => Action::NonReversible,
        }
    }
}
