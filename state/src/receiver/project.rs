use crate::receiver::ActionReceiver;
use crate::{Action, FloatField, IndexField, MultiTypeField, TypeField};
use shared::model::{
    EffectId, GeneratorId, Placement, PlacementId, Project, SampleId, TrackId,
};
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
            Action::AddChild(TypeField::Effect(effect)) => {
                let next_id = *self
                    .effects
                    .clone()
                    .into_keys()
                    .max()
                    .unwrap_or(EffectId(0))
                    + 1;
                let next_id = EffectId(next_id);
                self.effects.insert(next_id, effect.clone());
                Action::DeleteChildById(TypeField::EffectId(next_id))
            }
            Action::DeleteChildById(TypeField::EffectId(id)) => {
                let prev = self
                    .effects
                    .get(id)
                    .expect("Can't delete non-existent effect!")
                    .clone();
                self.effects.remove(id);
                Action::AddChild(TypeField::Effect(prev))
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
                let max_id = self.samples.keys().max().unwrap_or(&SampleId(0));
                self.samples
                    .insert(SampleId((**max_id) + 1), sample.clone());
                Action::NonReversible
            }
            Action::AddChild(TypeField::Track(track)) => {
                let max_id = self.tracks.keys().max().unwrap_or(&TrackId(0));
                let id = TrackId((**max_id) + 1);
                self.tracks.insert(id, track.clone());
                Action::DeleteChildById(TypeField::TrackId(id))
            }
            Action::DeleteChildById(TypeField::TrackId(id)) => {
                let prev = self
                    .tracks
                    .get(id)
                    .expect("Can't delete non-existent track")
                    .clone();
                self.tracks.remove(id);
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
            Action::AddChild(TypeField::MixerChannel(channel)) => {
                let new_index = self.mixer.channels.len();
                self.mixer.channels.push(channel.clone());
                self.mixer.matrix.add_channel(new_index);

                Action::DeleteChild(IndexField::Mixer(new_index))
            }
            Action::DeleteChild(IndexField::Mixer(index)) => {
                let prev = self.mixer.channels[*index].clone();
                self.mixer.channels.remove(*index);
                self.mixer.matrix.delete_channel(*index);
                Action::AddChild(TypeField::MixerChannel(prev))
            }
            Action::AddChild(TypeField::Generator(generator)) => {
                let all_ids: Vec<usize> = self
                    .generators
                    .keys()
                    .into_iter()
                    .map(|key| **key)
                    .collect();
                let next_id = if all_ids.is_empty() {
                    0
                } else {
                    *all_ids.iter().max().unwrap_or(&0) + 1
                };
                let next_id = GeneratorId(next_id);
                self.generators.insert(next_id, generator.clone());
                Action::DeleteChildById(TypeField::GeneratorId(next_id))
            }
            Action::DeleteChildById(TypeField::GeneratorId(id)) => {
                let prev = self
                    .generators
                    .get(id)
                    .expect("Can't delete non-existent generator!")
                    .clone();
                self.generators.remove(id);
                Action::AddChild(TypeField::Generator(prev))
            }
            // Action::AddChild(TypeField::DrumTrack(drum_track)) => {
            //     let id = if let Some(key) = self.drum_tracks.keys().max() {
            //         DrumTrackId((**key) + 1)
            //     } else {
            //         DrumTrackId(0)
            //     };
            //     self.drum_tracks.insert(id, drum_track.clone());
            //     Action::DeleteChildById(TypeField::DrumTrackId(id))
            // }
            _ => return None,
        })
    }
}
