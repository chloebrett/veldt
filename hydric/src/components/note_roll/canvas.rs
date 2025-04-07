use state::{Action, Selector, Store};

use egui::{
    Color32, Frame, Pos2, Rect, Response, ScrollArea, Sense, Shape, Ui, Vec2, emath::RectTransform,
    pos2, vec2,
};
use ordered_float::OrderedFloat;
use shared::{
    model::{Note, PitchName, PlacedNote, ScaleValue},
    types::PitchValue,
};

use super::{piano::Piano, roll::Roll};

use crate::{transform::Transform, widget::Sequencer};

// TODO Integrate into Store and project.
struct ProjectConfig {
    pub max_note: PitchValue,
    pub min_note: PitchValue,
    pub offset: f32,
    pub bars: f32,
    pub bar_length: f32,
}

pub fn note_roll_display(store: &Store, ui: &mut Ui) {
    let track_index = 0;
    new_note_button(store, ui, track_index);
    ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            draw_note_roll_canvas(store, ui, track_index);
        });
}

fn new_note_button(store: &Store, ui: &mut Ui, track_index: usize) {
    if ui.button("New note").clicked() {
        store.dispatch(
            &Selector::Track(track_index),
            Action::AddNote(PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: store.get().key,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: 0.0.into(),
            }),
        );
    }
}

fn draw_note_roll_canvas(store: &Store, ui: &mut Ui, track_index: usize) {
    let offset = 0.0;
    let bar_length = 4.0;
    let bars = 4.0;
    let max_note: PitchValue = PitchName {
        scale_value: ScaleValue::C,
        octave: 8,
    }
    .into();
    let min_note: PitchValue = PitchName {
        scale_value: ScaleValue::A,
        octave: 1,
    }
    .into();
    let project_config = ProjectConfig {
        max_note,
        min_note: min_note - 1,
        offset,
        bars,
        bar_length,
    };
    let piano_width = 50.0;
    let canvas_height = 600.0;
    let note_rects = make_all_note_rects(
        store.get().project.tracks[track_index].notes.clone(),
        max_note,
        offset,
    );
    let range = Rect::from_min_max(
        pos2(offset, min_note as f32 - 1.0),
        pos2(bars * bar_length, max_note as f32),
    );
    let dispatch = move |sel: &Selector, pos: Pos2| {
        let offset = pos.x;
        let pitch_name = PitchName::from(max_note - pos.y as i32);
        store.dispatch(sel, Action::SetNoteOffset(offset));
        store.dispatch(sel, Action::SetNoteOctave(pitch_name.octave));
        store.dispatch(sel, Action::SetNoteScaleValue(pitch_name.scale_value));
    };
    ui.add(
        Sequencer::new(range, dispatch)
            .rects(note_rects)
            .horizontal_rects(2.0, Color32::from_white_alpha(4))
            .vertical_bars(1.0, Color32::from_white_alpha(3))
            .vertical_bars(1.0 / bar_length, Color32::from_white_alpha(1)),
    );
    Frame::canvas(ui.style()).show(ui, |ui| {
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), canvas_height), Sense::hover());

        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let size = response.rect.size();
        let note_roll_canvas =
            NoteRollCanvas::new(store, project_config, size, piano_width, track_index);
        let shapes = note_roll_canvas.make_canvas_shapes();
        painter.extend(shapes.transform(to_screen));
        update_notes(
            ui,
            &response,
            to_screen,
            store,
            note_roll_canvas,
            track_index,
        );
        response
    });
}

struct NoteRollCanvas {
    piano: Piano,
    roll: Roll,
    project_config: ProjectConfig,
    piano_transform: RectTransform,
    roll_transform: RectTransform,
}

impl NoteRollCanvas {
    pub fn new(
        store: &Store,
        project_config: ProjectConfig,
        size: Vec2,
        piano_width: f32,
        track_index: usize,
    ) -> Self {
        let piano_transform = RectTransform::from_to(
            Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(
                    1.0,
                    (project_config.max_note - project_config.min_note) as f32,
                ),
            ),
            Rect::from_min_size(pos2(0.0, 0.0), vec2(piano_width, size.y)),
        );
        let roll_transform = RectTransform::from_to(
            Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(
                    project_config.bars * project_config.bar_length - project_config.offset,
                    (project_config.max_note - project_config.min_note) as f32,
                ),
            ),
            Rect::from_min_size(pos2(piano_width, 0.0), vec2(size.x - piano_width, size.y)),
        );
        NoteRollCanvas {
            piano: Piano::new(project_config.max_note, project_config.min_note),
            roll: Roll::new(
                store.get().project.tracks[track_index].notes.clone(),
                project_config.max_note,
                project_config.min_note,
                project_config.offset,
                project_config.bars,
                project_config.bar_length,
            ),
            project_config,
            piano_transform,
            roll_transform,
        }
    }

    pub fn make_canvas_shapes(&self) -> Vec<Shape> {
        // TODO Consider assigning render order values to shapes.
        let mut shapes = vec![];
        shapes.extend(self.roll.make_roll_shapes().transform(self.roll_transform));
        shapes.extend(
            self.piano
                .make_piano_shapes()
                .transform(self.piano_transform),
        );
        shapes
    }
}

pub fn note_to_pos(note: &PlacedNote, max_note: i32, project_offset: f32) -> Pos2 {
    let offset: f32 = note.offset.into();
    let x = offset - project_offset;
    let pitch_value: PitchValue = note.note.pitch_name.into();
    let y = max_note - pitch_value;
    pos2(x, y as f32)
}

pub fn make_note_rect(note: &PlacedNote, note_pos: Pos2) -> Rect {
    let note_size = vec2(note.note.beats, 1.0);
    Rect::from_min_size(note_pos, note_size)
}

pub fn make_all_note_rects(
    notes: Vec<PlacedNote>,
    max_note: i32,
    project_offset: f32,
) -> Vec<Rect> {
    notes
        .iter()
        .map(|note| {
            let note_pos = note_to_pos(note, max_note, project_offset);
            make_note_rect(note, note_pos)
        })
        .collect()
}

fn update_notes(
    ui: &Ui,
    response: &Response,
    to_screen: RectTransform,
    store: &Store,
    note_roll_canvas: NoteRollCanvas,
    track_index: usize,
) {
    store.get().project.tracks[track_index]
        .notes
        .iter()
        .enumerate()
        .for_each(|(note_index, note)| {
            let note_rect = make_note_rect(
                note,
                note_to_pos(
                    note,
                    note_roll_canvas.project_config.max_note,
                    note_roll_canvas.project_config.offset,
                ),
            );
            let note_response = track_note_response(
                ui,
                response,
                note_index,
                note_rect
                    .transform(note_roll_canvas.roll_transform)
                    .transform(to_screen),
            );
            let drag_pos = note_response.interact_pointer_pos();
            if let Some(pos) = drag_pos {
                let scaled_pos = pos
                    .transform(to_screen.inverse())
                    .transform(note_roll_canvas.roll_transform.inverse())
                    .clamp(
                        pos2(0.0, 0.0),
                        note_roll_canvas.roll_transform.from().size().to_pos2(),
                    );
                let offset: OrderedFloat<f32> = scaled_pos.x.into();
                let pitch_name =
                    PitchName::from(note_roll_canvas.project_config.max_note - scaled_pos.y as i32);
                dispatch_note(store, note_index, offset, pitch_name, track_index);
            }
        });
}

fn track_note_response(
    ui: &Ui,
    response: &Response,
    note_index: usize,
    note_rect: Rect,
) -> Response {
    let shape_index = response.id.with(note_index);
    ui.interact(note_rect, shape_index, Sense::drag())
}

fn dispatch_note(
    store: &Store,
    note_index: usize,
    offset: OrderedFloat<f32>,
    pitch_name: PitchName,
    track_index: usize,
) {
    let curr_note = &store.get().project.tracks[track_index].notes[note_index];
    let sel = Selector::Note(track_index, note_index);
    if curr_note.offset != offset {
        store.dispatch(&sel, Action::SetNoteOffset(offset.into()))
    };
    if curr_note.note.pitch_name.octave != pitch_name.octave {
        store.dispatch(&sel, Action::SetNoteOctave(pitch_name.octave))
    };
    if curr_note.note.pitch_name.scale_value != pitch_name.scale_value {
        store.dispatch(&sel, Action::SetNoteScaleValue(pitch_name.scale_value));
    }
}
