use crate::receiver::ActionReceiver;
use crate::{Action, FloatField};
use shared::model::MatrixCell;


impl ActionReceiver for MatrixCell {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::ModFactor, mod_factor) => {
                let prev: f32 = (*self).into();
                self.set(*mod_factor);
                Action::SetFloat(FloatField::ModFactor, prev)
            }
            _ => return None,
        })
    }
}
