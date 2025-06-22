use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MoveField, TypeField};
use shared::model::MixerChannel;

use super::move_elem;

impl ActionReceiver for MixerChannel {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::SetFloat(FloatField::Volume, volume) => {
                let prev = self.volume;
                self.volume = *volume;
                Action::SetFloat(FloatField::Volume, prev)
            }
            Action::MoveChild(MoveField {
                from_field,
                to_field,
            }) => {
                let (IndexField::EffectId(from), IndexField::EffectId(to)) = (from_field, to_field)
                else {
                    panic!("Action should have only received EffectId IndexFields.")
                };
                let prev = MoveField {
                    from_field: to_field.clone(),
                    to_field: from_field.clone(),
                };
                move_elem(&mut self.effect_ids, *from, *to);
                Action::MoveChild(prev)
            }
            Action::DeleteChildById(TypeField::EffectId(effect_id)) => {
                let index = self.effect_ids.iter().position(|it| it == effect_id)?;
                self.effect_ids.remove(index);
                Action::AddChildAtIndex(
                    TypeField::EffectId(*effect_id),
                    IndexField::EffectId(index),
                )
            }
            Action::AddChildAtIndex(
                TypeField::EffectId(effect_id),
                IndexField::EffectId(index),
            ) => {
                self.effect_ids.insert(*index, *effect_id);
                Action::DeleteChildById(TypeField::EffectId(*effect_id))
            }
            _ => return None,
        })
    }
}
