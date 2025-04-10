use egui::{pos2, vec2, Pos2, Rect, Ui};
use ordered_float::OrderedFloat;
use shared::{logger, model::Track};
use state::{Action, Store};

use crate::{view::View, widget::{Sequencer, SequencerObject}};

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
        let dispatch = |index: usize, action: Action| {};
        ui.add(Sequencer::new(range, dispatch).objects(tracks));
    }
}

impl SequencerObject<Track> for Track {
    fn to_pos(&self, range: Rect) -> Pos2 {
        // TODO handling channels. Currently all are at `y=1`.
        let y = 1.0;
        let offsets: Vec<OrderedFloat<f32>> = self.notes.iter().map(|note| note.offset).collect();
        let offset: f32 = offsets.into_iter()
            .max_by(|x, y| x.cmp(y))
            .unwrap_or(OrderedFloat(0.0)).into();
        let x = offset - range.left();
        pos2(x, y-1.0)
    }

    fn to_rect(&self, range: Rect) -> Rect {
        let track_pos = self.to_pos(range);
        let lengths: Vec<OrderedFloat<f32>> = self.notes.iter().map(|note| {
            note.offset + OrderedFloat(note.note.beats)
        }).collect();
        let max_length: f32 = lengths.into_iter()
            .max_by(|x, y| x.cmp(&y))
            .unwrap_or(OrderedFloat(1.0)).into();
        let track_size = vec2(max_length - track_pos.x, 1.0); 
        Rect::from_min_size(track_pos, track_size)
    } 

    fn x_action(&self, x: f32, range: Rect) -> Action {
        // TODO implement for track
        // This is a placeholder to satisfy trait
        Action::SetNoteOffset(x - range.left())
    }

    fn y_action(&self, y: f32, range: Rect) -> Action {
        // TODO implement for track
        // This is a placeholder to satisfy trait
        Action::SetNoteOffset(y)
    }

}
