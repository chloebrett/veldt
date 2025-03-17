use super::app::App;
use crate::state::Action;
use crate::widget::selectable_value;
use egui::{Context, Ui};
use mesic::create_scale_values;
use ordered_float::OrderedFloat;
use shared::model::{Note, PitchName, PlacedNote};
use shared::types::{Beats, Octave};

pub fn notes_control(app: &App, ui: &mut Ui, ctx: &Context) {
    let scale_options = create_scale_values(app.store.get().scale, app.store.get().key);

    for i in 0..app.store.get().project.tracks[0].notes.len() {
        egui::ComboBox::from_id_salt(i)
            .selected_text(
                app.store.get().project.tracks[0].notes[i]
                    .note
                    .pitch_name
                    .scale_value
                    .to_string(),
            )
            .show_ui(ui, |ui| {
                for scale_note in scale_options.iter() {
                    let scale_value = app.store.get().project.tracks[0].notes[i]
                        .note
                        .pitch_name
                        .scale_value;
                    selectable_value(
                        ui,
                        |it| {
                            it.map(|it| {
                                app.store.dispatch(Action::SetNoteScaleValue {
                                    track_index: 0,
                                    note_index: i,
                                    note: *it,
                                })
                            });
                            &scale_value
                        },
                        scale_note,
                        scale_note.to_string(),
                    );
                }
            });

        let octave = app.store.get().project.tracks[0].notes[i]
            .note
            .pitch_name
            .octave as f64;
        ui.add(
            egui::Slider::from_get_set(0.0..=8.0, |it| {
                it.map(|it| {
                    if it != octave {
                        app.store.dispatch(Action::SetNoteOctave {
                            track_index: 0,
                            note_index: i,
                            octave: it as Octave,
                        })
                    }
                });
                octave
            })
            .text("Octave")
            .fixed_decimals(0),
        );

        let duration = app.store.get().project.tracks[0].notes[i].note.beats as f64;
        ui.add(
            egui::Slider::from_get_set(0.0..=10.0, |it| {
                it.map(|it| {
                    if it != duration {
                        app.store.dispatch(Action::SetNoteDuration {
                            track_index: 0,
                            note_index: i,
                            duration: it as Beats,
                        });
                    }
                });
                app.store.get().project.tracks[0].notes[i].note.beats as f64
            })
            .text("Beats")
            .fixed_decimals(0),
        );

        let offset = *app.store.get().project.tracks[0].notes[i].offset as f64;
        ui.add(
            egui::Slider::from_get_set(0.0..=16.0, |it| {
                it.map(|it| {
                    if it != offset {
                        app.store.dispatch(Action::SetNoteOffset {
                            track_index: 0,
                            note_index: i,
                            offset: it as f32,
                        });
                    }
                });
                offset
            })
            .text("Offset"),
        );

        if ui.button("Delete").clicked() {
            app.store.dispatch(Action::DeleteNote {
                track_index: 0,
                note_index: i,
            });
            ctx.request_discard("");
            break;
        }
    }

    let track_length = app.store.get().project.tracks[0]
        .notes
        .clone()
        .into_iter()
        .map(|it| it.offset + it.note.beats)
        .max()
        .unwrap_or(OrderedFloat(0.0));

    if ui.button("New note").clicked() {
        app.store.dispatch(Action::AddNote {
            track_index: 0,
            note: PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: app.store.get().key,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: track_length,
            },
        });
    }
}
