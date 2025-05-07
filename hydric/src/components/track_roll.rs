use crate::{
    GetSet, LocalState, WindowState,
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
    Action, FloatField, IndexField, MultiIndexField, PlacementSelector, SelectorTrait, Store,
    TrackSelector, TypeField,
};
use std::collections::HashSet;

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
        let mut select = self.local_state.track_roll_select_enabled.get();
        if !select {
            self.local_state
                .selected_track_placements
                .set(HashSet::default());
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
        self.local_state.track_roll_select_enabled.set(select);
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

    fn get_active(_ui: &Ui, store: &Store, local_state: &LocalState) -> Option<PlacedTrack> {
        let index = local_state.active_track_placement.get()?;
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

    // TODO: remove all these unused _ui params.
    fn get_selected(_ui: &Ui, store: &Store, local_state: &LocalState) -> Vec<PlacedTrack> {
        local_state
            .selected_track_placements
            .get()
            .into_iter()
            .map(|index| {
                let placement_sel = PlacementSelector(index);
                let placement = store.select(&placement_sel);
                let track_placement: &TrackPlacement = placement.try_into().unwrap();
                PlacedTrack {
                    unclipped_duration: store
                        .select(&TrackSelector(track_placement.track_index))
                        .unclipped_duration(),
                    placement: placement.clone(),
                }
            })
            .collect()
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

    fn set_active(&self, _ui: &mut Ui, local_state: &LocalState, index: usize) {
        let track_placement: &TrackPlacement = (&self.placement).try_into().unwrap();

        local_state.note_roll_window.set(true);
        local_state.track_placement_window.set(true);
        local_state
            .active_track
            .set(Some(TrackSelector(track_placement.track_index)));
        local_state.active_track_placement.set(Some(index));
    }

    fn set_selected(_ui: &mut Ui, local_state: &LocalState, index: Option<usize>) {
        let Some(index) = index else {
            local_state
                .selected_track_placements
                .set(HashSet::default());
            return;
        };

        local_state.selected_track_placements.update(|mut it| {
            if it.contains(&index) {
                it.remove(&index);
            } else {
                it.insert(index);
            }
            it
        });
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

    fn delete_selected(
        _ui: &mut Ui,
        store: &Store,
        local_state: &LocalState,
        _parent_index: Option<usize>,
    ) {
        store.dispatchr(Action::DeleteChildren(MultiIndexField::Placement(
            local_state
                .selected_track_placements
                .get()
                .into_iter()
                .collect(),
        )));
    }
}
