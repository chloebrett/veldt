use crate::Action;
use crate::receiver::ActionReceiver;
use shared::model::Project;

impl ActionReceiver for Project {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddTrackPlacement(track_placement) => {
                let index = self.track_placements.len();
                self.track_placements.push(track_placement.clone());
                Action::DeleteTrackPlacement(index)
            }
            Action::DeleteTrackPlacement(track_placement_index) => {
                let prev = self
                    .track_placements
                    .get(*track_placement_index)
                    .expect("Can't delete non-existent track placement!")
                    .clone();
                self.track_placements.remove(*track_placement_index);
                Action::AddTrackPlacement(prev)
            }
            Action::SetProjectName(name) => {
                let prev = self.name.clone();
                self.name = name.to_string();
                Action::SetProjectName(prev)
            }
            Action::SetBpm(bpm) => {
                let prev = self.bpm;
                self.bpm = *bpm;
                Action::SetBpm(prev)
            }
            Action::AddSample(sample) => {
                self.samples.push(sample.clone());
                Action::NonReversible
            }
            Action::AddTrack(track) => {
                let prev = self.tracks.len();
                self.tracks.push(track.clone());
                Action::DeleteTrack(prev)
            }
            Action::DeleteTrack(track_index) => {
                let prev = self
                    .tracks
                    .get(*track_index)
                    .expect("Can't delete non-existent track")
                    .clone();
                self.tracks.remove(*track_index);
                Action::AddTrack(prev)
            }
            _ => return None,
        })
    }
}
