use super::{
    Action, StoreData, effect_index, effect_reducer, generator_index, generator_reducer,
    track_index, track_reducer,
};
use std::cell::RefMut;
use web_sys::console;

pub fn root_reducer(mut data: RefMut<'_, StoreData>, action: &Action) {
    console::log_1(&format!("root_reducer processing: {:?}", action.clone()).into());

    if let Some(track_index) = track_index(action) {
        return track_reducer(&mut data.project.tracks[track_index], action);
    }
    if let Some(generator_index) = generator_index(action) {
        return generator_reducer(&mut data.project.generators[generator_index], action);
    }
    if let Some((channel_index, effect_index)) = effect_index(action) {
        return effect_reducer(
            &mut data.project.mixer[channel_index].effects[effect_index],
            action,
        );
    }

    match action {
        Action::SetProjectName(name) => data.project.name = name.to_string(),
        Action::SetKey(key) => data.key = *key,
        Action::SetScale(scale) => data.scale = *scale,
        Action::SetBpm(bpm) => data.project.bpm = *bpm,
        Action::SetVolume(volume) => {
            console::log_1(&format!("old/new: {:?} {:?}", data.volume, volume).into());
            data.volume = *volume;
        }
        _ => {}
    }
}
