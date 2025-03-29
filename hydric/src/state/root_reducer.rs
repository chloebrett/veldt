use super::{
    Action, Selector, StoreData, effect_reducer, generator_reducer, note_reducer, track_reducer,
};
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
            Action::SetTrackList { tracks } => {
                data.track_list = tracks.clone();
                Action::NonReversible
            }
            Action::SetTrack { track_index, track } => {
                data.project.tracks[*track_index] = track.clone();
                Action::NonReversible
            }
            Action::SetLoadTrackName { track_name } => {
                data.load_track_name = Some(track_name.clone());
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
