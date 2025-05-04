use crate::receiver::ActionReceiver;
use crate::{Action, IndexField, MoveField, TypeField};
use shared::model::MixerChannel;

use super::move_elem;

impl ActionReceiver for MixerChannel {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::MoveChild(MoveField {
                from_field,
                to_field,
            }) => {
                let (IndexField::Effect(from), IndexField::Effect(to)) = (from_field, to_field)
                else {
                    panic!("Action should have only received Effect IndexFields.")
                };
                let prev = MoveField {
                    from_field: to_field.clone(),
                    to_field: from_field.clone(),
                };
                move_elem(&mut self.effects, *from, *to);
                Action::MoveChild(prev)
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
