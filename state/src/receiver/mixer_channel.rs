use crate::Action;
use crate::receiver::ActionReceiver;
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
            Action::DeleteEffect(effect_index) => {
                let prev = self.effects[*effect_index].clone();
                self.effects.remove(*effect_index);
                Action::AddEffect(prev)
            }
            Action::AddEffect(effect) => {
                self.effects.push(effect.clone());
                Action::DeleteEffect(self.effects.len() - 1)
            }
            _ => return None,
        })
    }
}
