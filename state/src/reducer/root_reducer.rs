use super::{
    effect_reducer, generator_reducer, mixer_reducer, note_reducer, track_placement_reducer,
    track_reducer,
};
use crate::{Action, Selector, StoreData};
use log::info;

pub fn root_reducer(data: &mut StoreData, selector: &Selector, action: &Action) -> Action {
    info!("root_reducer processing: {:?}", action.clone());

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
        Selector::Mixer(mixer_index) => {
            mixer_reducer(&mut data.project.mixer[*mixer_index], action)
        }
        Selector::Note(track_index, note_index) => note_reducer(
            &mut data.project.tracks[*track_index].notes[*note_index],
            action,
        ),
        Selector::TrackPlacement(track_placement_index) => track_placement_reducer(
            &mut data.project.track_placements[*track_placement_index],
            action,
        ),
        Selector::Root => match action {
            Action::AddTrackPlacement(track_placement) => {
                let index = data.project.track_placements.len();
                data.project.track_placements.push(track_placement.clone());
                Action::DeleteTrackPlacement(index)
            }
            Action::DeleteTrackPlacement(track_placement_index) => {
                let prev = data
                    .project
                    .track_placements
                    .get(*track_placement_index)
                    .expect("Can't delete non-existent track placement!")
                    .clone();
                data.project.track_placements.remove(*track_placement_index);
                Action::AddTrackPlacement(prev)
            }
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
            Action::SetProjectList(projects) => {
                data.project_list = projects.clone();
                Action::NonReversible
            }
            Action::SetProject(project) => {
                data.project = project.clone();
                Action::NonReversible
            }
            Action::SetLoadProjectName(project_name) => {
                data.load_project_name = Some(project_name.clone());
                Action::NonReversible
            }
            Action::AddSample(sample) => {
                data.project.samples.push(sample.clone());
                Action::NonReversible
            }
            Action::AddTrack(track) => {
                let prev = data.project.tracks.len();
                data.project.tracks.push(track.clone());
                Action::DeleteTrack(prev)
            }
            Action::DeleteTrack(track_index) => {
                let prev = data
                    .project
                    .tracks
                    .get(*track_index)
                    .expect("Can't delete non-existent track")
                    .clone();
                data.project.tracks.remove(*track_index);
                Action::AddTrack(prev)
            }
            _ => Action::NonReversible,
        },
    }
}
