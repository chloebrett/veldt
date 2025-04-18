use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, TypeField};
use shared::model::Project;

impl ActionReceiver for Project {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddChild(TypeField::TrackPlacement(track_placement)) => {
                let index = self.track_placements.len();
                self.track_placements.push(track_placement.clone());
                Action::DeleteChild(IndexField::TrackPlacement(index))
            }
            Action::DeleteChild(IndexField::TrackPlacement(index)) => {
                let prev = self
                    .track_placements
                    .get(*index)
                    .expect("Can't delete non-existent track placement!")
                    .clone();
                self.track_placements.remove(*index);
                Action::AddChild(TypeField::TrackPlacement(prev))
            }
            Action::SetChild(TypeField::ProjectName(name)) => {
                let prev = self.name.clone();
                self.name = name.to_string();
                Action::SetChild(TypeField::ProjectName(prev))
            }
            Action::SetFloat(FloatField::Bpm, bpm) => {
                let prev = self.bpm;
                self.bpm = *bpm;
                Action::SetFloat(FloatField::Bpm, prev)
            }
            Action::AddChild(TypeField::Sample(sample)) => {
                self.samples.push(sample.clone());
                Action::NonReversible
            }
            Action::AddChild(TypeField::Track(track)) => {
                let prev = self.tracks.len();
                self.tracks.push(track.clone());
                Action::DeleteChild(IndexField::Track(prev))
            }
            Action::DeleteChild(IndexField::Track(index)) => {
                let prev = self
                    .tracks
                    .get(*index)
                    .expect("Can't delete non-existent track")
                    .clone();
                self.tracks.remove(*index);
                Action::AddChild(TypeField::Track(prev))
            }
            _ => return None,
        })
    }
}
