use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::DelayConfig;

impl ActionReceiver for DelayConfig {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::DelayMs, delay_ms) => {
                let prev = self.delay_ms;
                self.delay_ms = *delay_ms;
                Action::SetFloat(FloatField::DelayMs, prev)
            }
            Action::SetFloat(FloatField::Feedback, feedback) => {
                let prev = self.feedback;
                self.feedback = *feedback;
                Action::SetFloat(FloatField::Feedback, prev)
            }
            _ => return None,
        })
    }
}
