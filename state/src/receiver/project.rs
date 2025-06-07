use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MultiIndexField, MultiTypeField, TypeField};
use shared::model::Project;

use super::delete_elems;

impl ActionReceiver for Project {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddChild(TypeField::Placement(placement)) => {
                let current_max_id = self.placements.iter().max_by(|(id, _) id);
                self.placements.insert(PlacementId(current_max_id+1), placement.clone());
                Action::DeleteChild(IndexField::Placement(index))
            }
            Action::DeleteChild(TypeField::PlacementId(id)) => {
                let prev = self
                    .placements
                    .get(id)
                    .expect("Can't delete non-existent track placement!")
                    .clone();
                self.placements.remove(id);
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
                let prev = self.placements.into_values().collect();
                delete_elems(&mut self.placements, indexes.clone());
                Action::SetChildren(MultiTypeField::Placement(prev))
            }
            Action::SetChildren(MultiTypeField::Placement(placements)) => {
                let prev = self.placements.into_values().collect();
                // TODO: just pass around the IDs in the multi type field (or make a new
                // KeyedTypeField with a map, or something).
                let mut id = 0;
                self.placements = HashMap::new();
                for placement in placements {
                    self.placements.insert(PlacementId(id), it);
                    id += 1;
                }
                Action::SetChildren(MultiTypeField::Placement(prev))
            }
            Action::AddChildren(MultiTypeField::Placement(placements)) => {
                // TODO: we need to be smarter about IDs here, potentially tracking IDs as part of
                // the action. Otherwise, we risk ID references going out of sync when 
                let prev: Vec<Placement> = self.placements.values().collect();
                let mut id = self.placements.iter().max_by(|(id, _) id);
                for placement in placements {
                    id += 1;
                    self.placements.insert(PlacementId(id), placement.clone());
                }
                Action::AddChildren(MultiTypeField::Placement(prev))
            }
            _ => return None,
        })
    }
}
