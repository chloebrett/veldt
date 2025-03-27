use super::{
    Action, Selector, StoreData, effect_reducer, generator_reducer, note_reducer, track_reducer,
};
use crate::rpc::{load_sample, load_track, load_track_list, save_track};
use poll_promise::Promise;
use std::rc::Rc;
use shared::logger::log;

pub fn root_reducer(data: &mut StoreData, selector: &Selector, action: &Action) -> Action {
    log(&format!("root_reducer processing: {:?}", action.clone()));

    match selector {
        Selector::Track(track_index) => {
            track_reducer(&mut data.project.tracks[*track_index], action)
        }
        Selector::Generator(generator_index) => {
            generator_reducer(&mut data.project.generators[*generator_index], action)
        }
        Selector::Effect(mixer_index, effect_index) => effect_reducer(
            &mut data.project.mixer[*mixer_index].effects[*effect_index],
            action,
        ),
        Selector::Note(track_index, note_index) => note_reducer(
            &mut data.project.tracks[*track_index].notes[*note_index],
            action,
        ),
        Selector::Root => match action {
            Action::SetProjectName(name) => {
                let prev = data.project.name.clone();
                data.project.name = name.to_string();
                Action::SetProjectName(prev)
            }
            Action::SetKey(key) => {
                let prev = data.key;
                data.key = *key;
                Action::SetKey(prev)
            }
            Action::SetScale(scale) => {
                let prev = data.scale;
                data.scale = *scale;
                Action::SetScale(prev)
            }
            Action::SetBpm(bpm) => {
                let prev = data.project.bpm;
                data.project.bpm = *bpm;
                Action::SetBpm(prev)
            }
            Action::SetVolume(volume) => {
                let prev = data.volume;
                data.volume = *volume;
                Action::SetVolume(prev)
            }
            Action::SaveTrack { track_index } => {
                let track_name = data.project.name.clone();
                let track = data.project.tracks[*track_index].clone();
                data.save_track_promise = Rc::new(Some(Promise::spawn_local(async move {
                    save_track(track_name, track).await
                })));
                Action::NonReversible
            }
            Action::LoadTrackList => {
                data.track_list_promise = Rc::new(Some(Promise::spawn_local(async move {
                    load_track_list().await
                })));
                Action::NonReversible
            }
            Action::SetTrackList { tracks } => {
                data.track_list = tracks.clone();
                Action::NonReversible
            }
            Action::SetTrack { track_index, track } => {
                data.project.tracks[*track_index] = track.clone();
                Action::NonReversible
            }
            Action::LoadTrack => {
                if let Some(name) = data.load_track_name.clone() {
                    data.load_track_promise =
                        Rc::new(Some(Promise::spawn_local(
                            async move { load_track(name).await },
                        )));
                }
                Action::NonReversible
            }
            Action::SetLoadTrackName { track_name } => {
                data.load_track_name = Some(track_name.clone());
                Action::NonReversible
            }
            Action::ClearLoadTrackPromise => {
                data.load_track_promise = Rc::new(None);
                Action::NonReversible
            }
            Action::ClearSaveTrackPromise => {
                data.save_track_promise = Rc::new(None);
                Action::NonReversible
            }
            Action::ClearTrackListPromise => {
                data.track_list_promise = Rc::new(None);
                Action::NonReversible
            }
            Action::LoadSample { filename } => {
                let filename = filename.clone();
                data.load_sample_promise = Rc::new(Some(Promise::spawn_local(async move {
                    load_sample(filename).await
                })));
                Action::NonReversible
            }
            Action::ClearLoadSamplePromise => {
                data.load_sample_promise = Rc::new(None);
                Action::NonReversible
            }
            Action::AddSample(sample) => {
                data.project.samples.push(sample.clone());
                Action::NonReversible
            }
            _ => Action::NonReversible,
        },
    }
}
