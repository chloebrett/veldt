use crate::receiver::ActionReceiver;
use crate::{Action, IndexField, MoveField, TypeField};
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
            Action::MoveChild(MoveField {
                from_field,
                to_field,
            }) => {
                if let (IndexField::Effect(from), IndexField::Effect(to)) = (from_field, to_field) {
                    let prev = MoveField {
                        from_field: to_field.clone(),
                        to_field: from_field.clone(),
                    };
                    self.effects.insert(*to, self.effects[*from].clone());
                    if from > to {
                        // Inserted value has increased original index of value by 1
                        self.effects.remove(from + 1);
                    } else if from <= to {
                        // Inserted value has not changed original index of value
                        self.effects.remove(*from);
                    };
                    Action::MoveChild(prev)
                } else {
                    panic!("Action should have only received Effect IndexFields.")
                }
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
