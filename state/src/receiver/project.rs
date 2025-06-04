use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MultiIndexField, MultiTypeField, TypeField};
use shared::model::Project;

use super::delete_elems;

impl ActionReceiver for Project {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddChild(TypeField::Placement(placement)) => {
                let index = self.placements.len();
                self.placements.push(placement.clone());
                Action::DeleteChild(IndexField::Placement(index))
            }
            Action::DeleteChild(IndexField::Placement(index)) => {
                let prev = self
                    .placements
                    .get(*index)
                    .expect("Can't delete non-existent track placement!")
                    .clone();
                self.placements.remove(*index);
                Action::AddChild(TypeField::Placement(prev))
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
            Action::DeleteChildren(MultiIndexField::Placement(indexes)) => {
                let prev = self.placements.clone();
                delete_elems(&mut self.placements, indexes.clone());
                Action::SetChildren(MultiTypeField::Placement(prev))
            }
            Action::SetChildren(MultiTypeField::Placement(placements)) => {
                let prev = self.placements.clone();
                self.placements = placements.to_vec();
                Action::SetChildren(MultiTypeField::Placement(prev))
            }
            Action::AddChildren(MultiTypeField::Placement(placements)) => {
                let prev = self.placements.clone();
                self.placements.extend(placements.to_vec());
                Action::AddChildren(MultiTypeField::Placement(prev))
            }
            _ => return None,
        })
    }
}
