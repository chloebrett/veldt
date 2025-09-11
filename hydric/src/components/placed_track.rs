use crate::components::utils::{ToEguiColour, choose_black_white_based_on_contrast};
use crate::window_state::WindowKind;
use crate::{GetSet, LocalState, transform::Transform};
use egui::epaint::{RectShape, TextShape};
use egui::{
    Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind, Ui, Vec2, emath::RectTransform,
    pos2, vec2,
};
use mesic::{beats_to_samples, samples_to_beats};
use ordered_float::OrderedFloat;
use shared::{
    model::{PlacedNote, Placement, PlacementId, PlacementType, SampleId, TrackPlacement},
    types::Beats,
};
use state::{Action, MultiTypeField, PlacementSelector, Store, TrackSelector, TypeField};
use std::cmp::{max, min};
use std::collections::HashSet;

const PITCH_RANGE: f32 = 4131.0;
const VISUAL_SAMPLING_RATE: usize = 120;

pub struct PlacedTrack<'a> {
    pub placement: Placement,
    pub unclipped_duration: OrderedFloat<f32>,
    pub store: &'a Store,
    pub local_state: &'a LocalState,
}

impl<'a> PlacedTrack<'a> {
    fn to_pos(&self, range: Rect) -> Pos2 {
        let y = self.placement.visual_placement as f32;
        let x = *self.placement.offset - range.left();
        pos2(x, y)
    }

    pub fn to_rect(&self, range: Rect) -> Rect {
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

    pub fn resize_action(&self, x: f32) -> Action {
        let clipped_duration = x - *self.placement.offset;
        let max_note_length = *self.unclipped_duration;
        let clipped_duration = if clipped_duration < max_note_length {
            Some(clipped_duration as Beats)
        } else {
            None
        };
        Action::SetChild(TypeField::ClippedDuration(clipped_duration))
    }

    pub fn shape(&self, range: Rect) -> Shape {
        let rgb = self.placement.colour;
        let background_colour = rgb.to_egui_unmultiplied(/* alpha= */ 20);
        Shape::Vec(vec![
            // track background shape
            Shape::rect_filled(
                self.to_rect(range),
                CornerRadius::same(1),
                background_colour,
            ),
            match &self.placement.kind {
                PlacementType::Track(track_placement) => {
                    let track_id = track_placement.track_id;
                    if let Some(track) = self.store.get().project.tracks.get(&track_id) {
                        let notes = &track.notes;
                        self.map_notes_to_shapes(range, notes)
                    } else {
                        Shape::rect_filled(
                            self.to_rect(range),
                            CornerRadius::same(1),
                            background_colour,
                        )
                    }
                }
                PlacementType::Sample(sample_placement) => {
                    let sample_id = sample_placement.sample_id;
                    self.sample_shape(range, sample_id)
                }
                PlacementType::DrumTrack(_) => {
                    // TODO implement drum track shape
                    Shape::rect_filled(
                        self.to_rect(range),
                        CornerRadius::same(1),
                        background_colour,
                    )
                }
            },
        ])
    }

    fn sample_shape(&self, range: Rect, sample_id: SampleId) -> Shape {
        if let Some(sample) = self.store.get().project.samples.get(&sample_id) {
            let sample_length = min(
                max(sample.left.len(), sample.right.len()),
                beats_to_samples(
                    *self
                        .placement
                        .clipped_duration
                        .unwrap_or(self.unclipped_duration),
                    self.store.get().project.bpm,
                ) as usize,
            );
            let colour = self.placement.colour.to_egui_additive();
            if sample_length == 0 {
                return Shape::line(vec![Pos2::ZERO], Stroke::new(0.0, colour));
            }
            let sample_placement_rect =
                Rect::from_x_y_ranges(0.0..=(sample_length - 1) as f32, 1.0..=-1.0);
            let sample_transform = RectTransform::from_to(
                sample_placement_rect,
                self.to_rect(range).shrink2(Vec2::new(0.0, 0.1)),
            );

            let mut points: Vec<Pos2> = Vec::new();
            if let Some(cached_points) = self
                .local_state
                .sample_visual_preview_cache
                .borrow()
                .get(&sample_id)
            {
                points = cached_points.to_vec();
            } else {
                // Always include the first point
                points.push(pos2(0.0, (sample.left[0] + sample.right[0]) * 0.5));

                // Sample every VISUAL_SAMPLING_RATEth point for better performance
                for i in (1..sample_length - 1).step_by(VISUAL_SAMPLING_RATE) {
                    let x = i as f32;
                    let y = (sample.left[i] + sample.right[i]) * 0.5;
                    points.push(pos2(x, y));
                }

                // Always include the last point
                points.push(pos2(
                    (sample_length - 1) as f32,
                    (sample.left[sample_length - 1] + sample.right[sample_length - 1]) * 0.5,
                ));

                // Cache points into local state
                self.local_state
                    .sample_visual_preview_cache
                    .borrow_mut()
                    .insert(sample_id, points.clone());
            }

            points = points[0..(sample_length / VISUAL_SAMPLING_RATE)].to_vec();
            Shape::line(points.transform(sample_transform), Stroke::new(0.1, colour))
        } else {
            Shape::line(vec![Pos2::ZERO], Stroke::new(0.0, Color32::BLACK))
        }
    }

    fn map_notes_to_shapes(&self, range: Rect, notes: &[PlacedNote]) -> Shape {
        let rgb_values = self.placement.colour;
        let note_positions: Vec<Pos2> = notes
            .iter()
            .map(|note| {
                let x_pos: f32 = note.offset.into();
                let y_pos: f32 = note.note.pitch_name.into();
                Pos2::new(x_pos, PITCH_RANGE - y_pos)
            })
            .collect();

        // placement_rect is the untransformed rect plane the notes will be initially mapped onto
        let placement_rect = Rect::from_min_max(
            Pos2::ZERO,
            Pos2::new(
                *self
                    .placement
                    .clipped_duration
                    .unwrap_or(self.unclipped_duration),
                PITCH_RANGE,
            ),
        );

        let note_rects: Vec<Rect> = notes
            .iter()
            .enumerate()
            .map(|(i, note)| {
                let mut note_rect = Rect::from_pos(note_positions[i]);
                note_rect.set_width(note.note.beats);
                note_rect.set_height(150.0);
                note_rect
            })
            .filter(|note_rect| {
                placement_rect.intersects(*note_rect) // placement may be clipped so only render notes that intersect with placement_rect 
            })
            .map(|mut note_rect| {
                let allowable_width = placement_rect.right_bottom().x - note_rect.left_bottom().x;
                let width = note_rect.width().min(allowable_width); // shrink note rect if it exceeds the bounds of the placement_rect
                note_rect.set_width(width);
                note_rect
            })
            .collect();

        let note_shapes: Vec<Shape> = note_rects
            .into_iter()
            .map(|note_rect| {
                let note_shape: Shape = RectShape::new(
                    note_rect,
                    1.0,
                    rgb_values.to_egui_additive(),
                    Stroke::NONE,
                    StrokeKind::Inside,
                )
                .into();
                note_shape
            })
            .collect();

        let track_transform = RectTransform::from_to(
            placement_rect,
            self.to_rect(range).shrink2(Vec2::new(0.0, 0.07)),
        );
        Shape::Vec(note_shapes.transform(track_transform))
    }

    pub fn create_header_shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        let header_brackground_colour = rgb_values.to_egui_additive();

        let mut header_rect = Rect::from_pos(Pos2::ZERO);
        let duration = *self
            .placement
            .clipped_duration
            .unwrap_or(self.unclipped_duration);
        header_rect.set_width(duration);
        header_rect.set_height(500.0);

        let placement_transform = RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(duration, PITCH_RANGE)),
            self.to_rect(range),
        );

        Shape::rect_filled(header_rect, 0.2, header_brackground_colour)
            .transform(placement_transform)
    }

    pub fn create_label(&self, range: Rect, ui: &mut Ui) -> Shape {
        let font_id = egui::FontId::proportional(11.0);
        let font_rgb = choose_black_white_based_on_contrast(self.placement.colour);
        let font_colour = font_rgb.to_egui();
        let label_text = match &self.placement.kind {
            PlacementType::Track(track) => "Track: ".to_owned() + &track.track_id.to_string(),
            PlacementType::Sample(sample) => "Sample: ".to_owned() + &sample.sample_id.to_string(),
            PlacementType::DrumTrack(drum) => "Drum: ".to_owned() + &drum.track_id.to_string(),
        };
        let galley =
            ui.fonts(|fonts| fonts.layout_no_wrap(label_text, font_id.clone(), font_colour));
        let transform = RectTransform::from_to(
            Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(
                    *self
                        .placement
                        .clipped_duration
                        .unwrap_or(self.unclipped_duration),
                    PITCH_RANGE,
                ),
            ),
            self.to_rect(range),
        );
        let position = Pos2::ZERO.transform(transform);
        let text_shape = Shape::Text(TextShape {
            pos: position,
            galley,
            underline: Stroke::NONE,
            override_text_color: Some(font_colour),
            angle: 0.0,
            fallback_color: font_colour,
            opacity_factor: 1.0,
        });
        Shape::Vec(vec![self.create_header_shape(range), text_shape])
    }

    pub fn get_active(store: &'a Store, local_state: &'a LocalState) -> Option<PlacedTrack<'a>> {
        let id = local_state.active_placement.get()?;
        let selector = PlacementSelector(id);
        let placement = store.select(&selector);

        let unclipped_duration = match &placement.kind {
            PlacementType::Track(track_placement) => store
                .select(&TrackSelector(track_placement.track_id))
                .unclipped_duration(),
            PlacementType::Sample(sample_placement) => {
                if let Some(sample) = store.get().project.samples.get(&sample_placement.sample_id) {
                    ordered_float::OrderedFloat(samples_to_beats(
                        max(sample.left.len(), sample.right.len()),
                        store.get().project.bpm,
                    ))
                } else {
                    ordered_float::OrderedFloat(1.0)
                }
            }
            PlacementType::DrumTrack(_) => {
                // TODO calculate unclipped duration
                OrderedFloat(8.0)
            }
        };

        Some(PlacedTrack {
            unclipped_duration,
            placement: placement.clone(),
            store,
            local_state,
        })
    }

    pub fn active_shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.5,
                    color: rgb_values.to_egui_additive(),
                },
                StrokeKind::Inside,
            ),
        ])
    }

    pub fn get_selected(store: &'a Store, local_state: &'a LocalState) -> Vec<PlacedTrack<'a>> {
        local_state
            .selected_placements
            .get()
            .into_iter()
            .map(|index| {
                let placement_sel = PlacementSelector(index);
                let placement = store.select(&placement_sel);
                let track_placement: &TrackPlacement = placement.try_into().unwrap();
                PlacedTrack {
                    unclipped_duration: store
                        .select(&TrackSelector(track_placement.track_id))
                        .unclipped_duration(),
                    placement: placement.clone(),
                    store,
                    local_state,
                }
            })
            .collect()
    }

    pub fn selected_shape(&self, range: Rect) -> Shape {
        let rgb_values = self.placement.colour;
        Shape::Vec(vec![
            self.shape(range),
            Shape::rect_stroke(
                self.to_rect(range),
                CornerRadius::same(0),
                Stroke {
                    width: 1.5,
                    color: rgb_values.to_egui_additive(),
                },
                StrokeKind::Inside,
            ),
        ])
    }

    pub fn set_active(&self, local_state: &LocalState, id: PlacementId) {
        match &self.placement.kind {
            PlacementType::DrumTrack(drum_track_placement) => {
                local_state
                    .active_track
                    .set(Some(TrackSelector(drum_track_placement.track_id)));
            }
            PlacementType::Track(track_placement) => {
                local_state
                    .active_track
                    .set(Some(TrackSelector(track_placement.track_id)));
            }
            _ => (),
        }

        local_state
            .window_state
            .set_visible(WindowKind::NoteRoll, true);
        local_state
            .window_state
            .set_visible(WindowKind::Placement, true);

        local_state.active_placement.set(Some(id));
    }

    pub fn set_selected(local_state: &LocalState, id: Option<PlacementId>) {
        // TODO: weird API. Deleting all selections if this is none? Shouldn't we just pass a set
        // of IDs?
        let Some(id) = id else {
            local_state.selected_placements.set(HashSet::default());
            return;
        };

        local_state.selected_placements.update(|mut it| {
            if it.contains(&id) {
                it.remove(&id);
            } else {
                it.insert(id);
            }
            it
        });
    }

    pub fn delete_selected(store: &Store, local_state: &LocalState) {
        local_state
            .active_placement
            .update(|placement| match placement {
                // If the active placement is selected, "de-activate" it.
                Some(id) if local_state.selected_placements.get().contains(&id) => None,
                _ => placement,
            });
        store.dispatchr(Action::DeleteChildrenById(MultiTypeField::PlacementId(
            local_state.selected_placements.get().into_iter().collect(),
        )));
    }

    pub fn delete_self(&self, store: &Store, local_state: &LocalState, placement_id: PlacementId) {
        local_state.active_placement.update(|placement| {
            if placement == Some(placement_id) {
                None
            } else {
                placement
            }
        });
        store.dispatchr(Action::DeleteChildById(TypeField::PlacementId(
            placement_id,
        )));
    }
}
