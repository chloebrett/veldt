use super::{
    Action, StoreData, effect_index, effect_reducer, generator_index, generator_reducer,
    track_index, track_reducer,
};
use crate::rpc::{load_track, load_track_list, save_track};
use poll_promise::Promise;
use std::cell::RefMut;
use std::rc::Rc;
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
        Action::SaveTrack { track_index } => {
            let track_name = data.project.name.clone();
            let track = data.project.tracks[*track_index].clone();
            data.save_track_promise = Rc::new(Some(Promise::spawn_local(async move {
                save_track(track_name, track).await
            })));
        }
        Action::LoadTrackList => {
            data.track_list_promise = Rc::new(Some(Promise::spawn_local(async move {
                load_track_list().await
            })));
        }
        Action::SetTrackList { tracks } => data.track_list = tracks.clone(),
        Action::SetTrack { track_index, track } => {
            data.project.tracks[*track_index] = track.clone()
        }
        Action::LoadTrack => {
            if let Some(name) = data.load_track_name.clone() {
                data.load_track_promise =
                    Rc::new(Some(Promise::spawn_local(
                        async move { load_track(name).await },
                    )))
            }
        }
        Action::SetLoadTrackName { track_name } => data.load_track_name = Some(track_name.clone()),
        Action::ClearLoadTrackPromise => data.load_track_promise = Rc::new(None),
        Action::ClearSaveTrackPromise => data.save_track_promise = Rc::new(None),
        Action::ClearTrackListPromise => data.track_list_promise = Rc::new(None),
        _ => {}
    }
}
