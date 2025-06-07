use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MultiTypeField, TypeField};
use shared::model::{Placement, PlacementId, Project};
use std::collections::HashMap;

impl ActionReceiver for Project {
    fn apply(&mut self, action: &Action) -> Option<Action> {
        Some(match action {
            Action::AddChild(TypeField::Placement(placement)) => {
                let next_id = *self
                    .placements
                    .clone()
                    .into_keys()
                    .max()
                    .unwrap_or(PlacementId(0))
                    + 1;
                let next_id = PlacementId(next_id);
                self.placements.insert(next_id, placement.clone());
                Action::DeleteChildById(TypeField::PlacementId(next_id))
            }
            Action::DeleteChildById(TypeField::PlacementId(id)) => {
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
            Action::DeleteChildrenById(MultiTypeField::PlacementId(ids)) => {
                let prev = self.placements.clone().into_values().collect();
                for id in ids {
                    self.placements.remove(id);
                }
                Action::SetChildren(MultiTypeField::Placement(prev))
            }
            Action::SetChildren(MultiTypeField::Placement(placements)) => {
                let prev = self.placements.clone().into_values().collect();
                // TODO: just pass around the IDs in the multi type field (or make a new
                // KeyedTypeField with a map, or something).
                self.placements = HashMap::new();
                for (id, placement) in placements.iter().enumerate() {
                    self.placements.insert(PlacementId(id), placement.clone());
                }
                Action::SetChildren(MultiTypeField::Placement(prev))
            }
            Action::AddChildren(MultiTypeField::Placement(placements)) => {
                // TODO: we need to be smarter about IDs here, potentially tracking IDs as part of
                // the action. Otherwise, we risk ID references going out of sync when performing
                // actions and undo/redo.
                let prev: Vec<Placement> = self.placements.clone().into_values().collect();
                let mut id = *self
                    .placements
                    .clone()
                    .into_keys()
                    .max()
                    .unwrap_or(PlacementId(0))
                    + 1;
                for placement in placements {
                    self.placements.insert(PlacementId(id), placement.clone());
                    id += 1;
                }
                Action::AddChildren(MultiTypeField::Placement(prev))
            }
            _ => return None,
        })
    }
}
