use crate::receiver::ActionReceiver;
use crate::{Action, IndexField, TypeField};
use shared::model::MixerChannel;

impl ActionReceiver for MixerChannel {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::MoveEffectDown(effect_index) => {
                self.effects.swap(*effect_index, effect_index + 1);
                Action::MoveEffectUp(*effect_index)
            }
            Action::MoveEffectUp(effect_index) => {
                self.effects.swap(*effect_index, effect_index - 1);
                Action::MoveEffectDown(*effect_index)
            }
            Action::DeleteChild(IndexField::Effect(effect_index)) => {
                let prev = self.effects[*effect_index].clone();
                self.effects.remove(*effect_index);
                Action::AddChild(TypeField::Effect(prev))
            }
            Action::AddChild(TypeField::Effect(effect)) => {
                self.effects.push(effect.clone());
                Action::DeleteChild(IndexField::Effect(self.effects.len() - 1))
            }
            _ => return None,
        })
    }
}
