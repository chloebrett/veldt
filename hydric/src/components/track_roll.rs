use egui::{Color32, Pos2, Rect, Ui, pos2, vec2};
use ordered_float::OrderedFloat;
use shared::model::{Track, TrackPlacement};
use state::{Action, Selector, Store};

use crate::{
    view::View,
    widget::{Sequencer, SequencerObject},
};

pub struct TrackRoll;

impl TrackRoll {
    pub fn new() -> Self {
        TrackRoll {}
    }
}

impl View for TrackRoll {
    fn ui(&self, store: &Store, ui: &mut Ui) {
        let mut placed_tracks = vec![];
        for placement in &store.get().project.track_placements {
            let track_index = placement.track_id;
            let placed_track = PlacedTrack {
                track: store.get().project.tracks[track_index].clone(),
                placement: placement.clone(),
            };
            placed_tracks.push(placed_track)
        }
        let range = Rect::from_min_max(pos2(0.0, 0.0), pos2(16.0, 1.0));
        let dispatch =
            |index: usize, action: Action| store.dispatch(&Selector::Track(index), action);
        ui.add(
            Sequencer::new(range, dispatch)
                .objects(placed_tracks)
                .size(vec2(ui.available_width(), 100.0))
                .vertical_bars(4.0, Color32::from_white_alpha(6))
                .vertical_bars(1.0, Color32::from_white_alpha(3)),
        );
    }
}

struct PlacedTrack {
    track: Track,
    placement: TrackPlacement,
}

impl SequencerObject<PlacedTrack> for PlacedTrack {
    fn to_pos(&self, range: Rect) -> Pos2 {
        // TODO handling multiple channels. Currently all are at `y=0`.
        let y = 0.0;
        let x = *self.placement.offset - range.left();
        pos2(x, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let track_pos = self.to_pos(range);
        // If not clipped duration render length based on notes.
        let length: f32 = if self.placement.clipped_duration.is_some() {
            self.placement
                .clipped_duration
                .expect("Expected a duration.")
                .into()
        } else {
            let lengths: Vec<OrderedFloat<f32>> = self
                .track
                .notes
                .iter()
                .map(|note| note.offset + OrderedFloat(note.note.beats))
                .collect();
            lengths
                .into_iter()
                .max_by(|x, y| x.cmp(y))
                .unwrap_or(OrderedFloat(1.0))
                .into()
        };
        let track_size = vec2(length, 1.0);
        Rect::from_min_size(track_pos, track_size)
    }

    fn x_action(&self, _x: f32, _range: Rect) -> Option<Action> {
        // TODO Consider if offset should be changable from sequencer.
        None
    }

    fn y_action(&self, _y: f32, _range: Rect) -> Option<Action> {
        // TODO implement multiple channels.
        None
    }

    fn resize_action(&self, _x: f32, _range: Rect) -> Option<Action> {
        // TODO Consider if clipped_duration should be changable from sequencer.
        None
    }

    fn is_interactable(&self) -> bool {
        false
    }
}
