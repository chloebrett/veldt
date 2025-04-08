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
        _ => Action::NonReversible,
    }
}
