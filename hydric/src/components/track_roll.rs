use egui::{Color32, Pos2, Rect, Ui, pos2, vec2};
use ordered_float::OrderedFloat;
use shared::model::Track;
use state::{Action, Store};

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
        let tracks = store.get().project.tracks.clone();
        let range = Rect::from_min_max(pos2(0.0, 0.0), pos2(16.0, 1.0));
        let dispatch = |_index: usize, _action: Action| {};
        ui.add(
            Sequencer::new(range, dispatch)
                .objects(tracks)
                .size(vec2(ui.available_width(), 100.0))
                .vertical_bars(4.0, Color32::from_white_alpha(6))
                .vertical_bars(1.0, Color32::from_white_alpha(3)),
        );
    }
}

impl SequencerObject<Track> for Track {
    fn to_pos(&self, range: Rect) -> Pos2 {
        // TODO handling multiple channels. Currently all are at `y=0`.
        let y = 0.0;
        let offsets: Vec<OrderedFloat<f32>> = self.notes.iter().map(|note| note.offset).collect();
        let offset: f32 = offsets
            .into_iter()
            .min_by(|x, y| x.cmp(y))
            .unwrap_or(OrderedFloat(0.0))
            .into();
        let x = offset - range.left();
        pos2(x, y)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let track_pos = self.to_pos(range);
        let lengths: Vec<OrderedFloat<f32>> = self
            .notes
            .iter()
            .map(|note| note.offset + OrderedFloat(note.note.beats))
            .collect();
        let max_length: f32 = lengths
            .into_iter()
            .max_by(|x, y| x.cmp(&y))
            .unwrap_or(OrderedFloat(1.0))
            .into();
        let track_size = vec2(max_length - track_pos.x, 1.0);
        Rect::from_min_size(track_pos, track_size)
    }

    fn x_action(&self, x: f32, range: Rect) -> Action {
        // TODO implement for track
        // This is a placeholder to satisfy trait
        Action::SetNoteOffset(x - range.left())
    }

    fn y_action(&self, y: f32, _range: Rect) -> Action {
        // TODO implement for track
        // This is a placeholder to satisfy trait
        Action::SetNoteOffset(y)
    }
}
