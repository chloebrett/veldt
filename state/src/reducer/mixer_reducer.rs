use crate::Action;
use shared::logger::log;
use shared::model::MixerChannel;

pub fn mixer_reducer(mixer: &mut MixerChannel, action: &Action) -> Action {
    log(&format!("mixer_reducer processing: {:?}", action.clone()));

    match action {
        Action::MoveEffectDown(effect_index) => {
            mixer.effects.swap(*effect_index, effect_index + 1);
            Action::MoveEffectUp(*effect_index)
        }
        Action::MoveEffectUp(effect_index) => {
            mixer.effects.swap(*effect_index, effect_index - 1);
            Action::MoveEffectDown(*effect_index)
        }
        Action::DeleteEffect(effect_index) => {
            let prev = mixer.effects[*effect_index].clone();
            mixer.effects.remove(*effect_index);
            Action::AddEffect(prev)
        }
        Action::AddEffect(effect) => {
            mixer.effects.push(effect.clone());
            Action::DeleteEffect(mixer.effects.len() - 1)
        }
        _ => Action::NonReversible,
    }
}
