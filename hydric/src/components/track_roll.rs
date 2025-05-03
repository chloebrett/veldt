use crate::{
    DataState, GetSetOption, LocalState, WindowState, update_select_data_state,
    view::View,
    widget::{Sequencer, SequencerObject, default_window},
};
use egui::{
    Color32, CornerRadius, Pos2, Rect, ScrollArea, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Placement, PlacementType, Track, TrackPlacement},
    types::Beats,
};
use state::{
    Action, FloatField, IndexField, PlacementSelector, SelectorTrait, Store, TrackSelector,
    TypeField,
};
use std::collections::BTreeSet;

pub struct TrackRoll<'a> {
    store: &'a Store,
    window_state: &'a mut WindowState,
    local_state: &'a LocalState,
}

impl<'a> TrackRoll<'a> {
    pub fn new(
        store: &'a Store,
        window_state: &'a mut WindowState,
        local_state: &'a LocalState,
    ) -> Self {
        Self {
            store,
            window_state,
            local_state,
        }
    }
}

impl View for TrackRoll<'_> {
    fn ui(&mut self, ui: &mut Ui) {
        let store = self.store;
        let placed_tracks: Vec<PlacedTrack> = store
            .get()
            .project
            .placements
            .iter()
            .map(|placement| {
                // TODO: handle sample placements.
                let track_placement: &TrackPlacement = placement.try_into().unwrap();

                PlacedTrack {
                    unclipped_duration: store.get().project.tracks[track_placement.track_index]
                        .unclipped_duration(),
                    placement: placement.clone(),
                }
            })
            .collect();
        let track_count = store.get().project.tracks.len();
        let range = Rect::from_min_max(pos2(0.0, 0.0), pos2(16.0, track_count as f32));
        let mut select = DataState::TrackRollSelectMode
            .get_value(ui)
            .unwrap_or(false);
        if !select {
            DataState::SelectedTrackPlacementIndexes.remove_value(ui);
        }
        default_window("Track Roll")
            .default_pos(pos2(30.0, 200.0))
            .resizable(true)
            .open(&mut self.window_state.track_roll)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New track").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Track(Track::default())));
                    }
                    if ui.button("New track placement").clicked() {
                        store.dispatchr(Action::AddChild(TypeField::Placement(
                            Placement::default(),
                        )));
                    }
                    ui.checkbox(&mut select, "Select")
                });
                ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            Sequencer::new(store, self.local_state, range)
                                .objects(placed_tracks)
                                .size(vec2(600.0, 100.0 * track_count as f32))
                                .select(select)
                                .vertical_bars(4.0, Color32::from_white_alpha(6))
                                .vertical_bars(1.0, Color32::from_white_alpha(3))
                                .horizontal_rects(
                                    |index| index % 2 == 1,
                                    Color32::from_white_alpha(1),
                                ),
                        );
                    });
            });
        DataState::TrackRollSelectMode.set_value(ui, select);
    }
}

struct PlacedTrack {
    placement: Placement,
    unclipped_duration: OrderedFloat<f32>,
}

impl SequencerObject<PlacedTrack> for PlacedTrack {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let track_placement: &TrackPlacement = (&self.placement).try_into().unwrap();

        // TODO handling multiple channels. Currently all are at `y=0`.
        let y = track_placement.track_index as f32;
        let x = *self.placement.offset - range.left();
        pos2(x, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let track_pos = self.to_pos(range);
        // If not clipped duration render length based on notes.
        let length: f32 = *self
            .placement
            .clipped_duration
            .unwrap_or(self.unclipped_duration);
        // Min `track_size.x` of 0.4 to ensure part of the object is still visible to interact with.
        let track_size = vec2(length.max(0.4), 1.0);
        Rect::from_min_size(track_pos, track_size)
    }

    fn x_action(&self, x: f32, range: Rect) -> Option<Action> {
        Some(Action::SetFloat(FloatField::Offset, x - range.left()))
    }

    fn y_action(&self, y: f32, _range: Rect) -> Option<Action> {
        // TODO implement multiple tracks.
        Some(Action::SetIndex(IndexField::Track(y as usize)))
    }

    fn resize_action(&self, x: f32, _range: Rect) -> Option<Action> {
        let clipped_duration = x - *self.placement.offset;
        let max_note_length = *self.unclipped_duration;
        let clipped_duration = if clipped_duration < max_note_length {
            Some(clipped_duration as Beats)
        } else {
            None
        };
        Some(Action::SetChild(TypeField::ClippedDuration(
            clipped_duration,
        )))
    }

    fn shape(&self, range: Rect) -> Shape {
        if *self.unclipped_duration == 0.0 {
            Shape::rect_filled(
                self.to_rect(range),
                CornerRadius::same(1),
                Color32::from_white_alpha(32),
            )
        } else {
            Shape::rect_filled(self.to_rect(range), CornerRadius::same(1), Color32::WHITE)
        }
    }

    fn get_active(ui: &Ui, store: &Store, _local_state: &LocalState) -> Option<PlacedTrack> {
        let index = DataState::ActiveTrackPlacementIndex.get_value::<usize>(ui)?;
        let selector = PlacementSelector(index);
        let placement = store.select(&selector);
        let track_placement: &TrackPlacement = placement.try_into().unwrap();
        Some(PlacedTrack {
            unclipped_duration: store
                .select(&TrackSelector(track_placement.track_index))
                .unclipped_duration(),
            placement: placement.clone(),
        })
    }

    fn active_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::BLUE,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn get_selected(ui: &Ui, store: &Store, _local_state: &LocalState) -> Option<Vec<PlacedTrack>> {
        let index_list: BTreeSet<usize> = DataState::SelectedTrackPlacementIndexes.get_value(ui)?;
        Some(
            index_list
                .into_iter()
                .map(|index| {
                    let placement = store
                        .get()
                        .project
                        .placements
                        .get(index)
                        .expect("Should have been track placement at index");
                    let track_placement: &TrackPlacement = placement.try_into().unwrap();
                    PlacedTrack {
                        unclipped_duration: store
                            .get()
                            .project
                            .tracks
                            .get(track_placement.track_index)
                            .expect("Should have been track at index.")
                            .unclipped_duration(),
                        placement: placement.clone(),
                    }
                })
                .collect(),
        )
    }

    fn selected_shape(&self, range: Rect) -> Shape {
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.0,
                    color: Color32::RED,
                },
                StrokeKind::Inside,
            ),
        ])
    }

    fn selector(index: usize, _parent_index: Option<usize>) -> impl SelectorTrait {
        PlacementSelector(index)
    }

    fn set_active(&self, ui: &mut Ui, local_state: &LocalState, index: usize) {
        let track_placement: &TrackPlacement = (&self.placement).try_into().unwrap();

        DataState::NoteRollWindow.set_value(ui, true);
        DataState::TrackPlacementViewWindow.set_value(ui, true);
        local_state
            .active_track
            .set(TrackSelector(track_placement.track_index));
        DataState::ActiveTrackPlacementIndex.set_value(ui, index);
    }

    fn set_selected(ui: &mut Ui, _local_state: &LocalState, index: Option<usize>) {
        update_select_data_state(ui, DataState::SelectedTrackPlacementIndexes, index);
    }

    fn add_new(&self, store: &Store, _parent_index: Option<usize>) {
        store.dispatchr(Action::AddChild(TypeField::Placement(
            self.placement.clone(),
        )));
    }

    fn from_pos(pos: Pos2, range: Rect) -> PlacedTrack {
        let track_index = pos.y as usize;
        let offset = range.left() + pos.x;
        PlacedTrack {
            placement: Placement {
                kind: PlacementType::Track(TrackPlacement {
                    track_index,
                    generator_index: 0,
                }),
                offset: offset.into(),
                clipped_duration: None,
                visual_placement: 0,
            },
            unclipped_duration: 0.0.into(),
        }
    }

    fn delete(store: &Store, index: usize, _parent_index: Option<usize>) {
        store.dispatchr(Action::DeleteChild(IndexField::Placement(index)));
    }

    fn delete_selected(
        ui: &mut Ui,
        store: &Store,
        _local_state: &LocalState,
        parent_index: Option<usize>,
    ) {
        // Track Placements must be deleted in reverse order so that indices for the rest of the selected
        // placements do not change mid-process. E.g., if deleting `3` and `4`, if `3` is deleted first
        // the placement that was at `4` will now be at `3` and the algorithm will either delete the wrong note or raise
        // and error.
        // BTreeSet provides an effecient way to keep and get from a sorted list.
        for index in DataState::SelectedTrackPlacementIndexes
            .get_value::<BTreeSet<usize>>(ui)
            .unwrap_or_default()
            .iter()
            .rev()
        {
            PlacedTrack::delete(store, *index, parent_index);
        }
    }
}
