use leptos::prelude::*;
use mesic::scale::create_scale_values;
use shared::model::{Note, PitchName, Scale, ScaleValue};
use shared::types::{Beats, Octave};
use std::str::FromStr;
use thaw::{Button, ButtonAppearance, Card, Select, Space, SpinButton, Tooltip};

#[derive(Clone)]
pub struct NoteSignal {
    id: u32,
    // Arc so that they get cleaned up when removed from the notes list.
    pub scale_value: ArcRwSignal<String>,
    pub octave: ArcRwSignal<Octave>,
    pub duration: ArcRwSignal<Beats>,
}

impl From<NoteSignal> for Note {
    fn from(item: NoteSignal) -> Note {
        Note {
            pitch_name: PitchName {
                scale_value: ScaleValue::from_str(&item.scale_value.get()).unwrap(),
                octave: item.octave.get(),
            },
            beats: item.duration.get(),
        }
    }
}

#[component]
pub fn NotesPanel(
    notes: RwSignal<Vec<NoteSignal>>,
    scale_string: RwSignal<String>,
    key_string: RwSignal<String>,
) -> impl IntoView {
    let next_note_id = RwSignal::new(notes.get_untracked().len() as u32);
    let scale_notes = move || {
        create_scale_values(
            Scale::from_str(&scale_string.get()).unwrap(),
            ScaleValue::from_str(&key_string.get()).unwrap(),
        )
    };
    let add_note = move |_| {
        let note = NoteSignal {
            id: next_note_id.get(),
            scale_value: ArcRwSignal::new(ScaleValue::A.to_string()),
            octave: ArcRwSignal::new(4),
            duration: ArcRwSignal::new(1.0),
        };

        notes.update(move |notes| notes.push(note));

        next_note_id.update(|it| *it += 1);
    };

    let delete_note = move |id| {
        notes.update(move |notes| notes.retain(|note| note.id != id));
    };

    view! {
        <Card>
            <Button appearance=ButtonAppearance::Secondary on_click=add_note>
                "Add note"
            </Button>
            <For
                each=move || notes.get()
                key=|note| note.id
                children=move |signal| {
                    let scale_value = RwSignal::from(signal.scale_value);
                    let octave = RwSignal::from(signal.octave);
                    let duration = RwSignal::from(signal.duration);

                    view! {
                        <Space>
                            <Tooltip content="Note">
                                <Select value=scale_value>
                                    <For
                                        each=scale_notes
                                        key=|value| value.to_string()
                                        children=move |value| {
                                            view! { <option>{value.to_string()}</option> }
                                        }
                                    />
                                </Select>
                            </Tooltip>
                            <Tooltip content="Octave">
                                <SpinButton<i32> value=octave step_page=1 min=0 max=8 />
                            </Tooltip>
                            <Tooltip content="Duration">
                                <SpinButton<f32> value=duration step_page=0.25 min=0.5 max=16.0 />
                            </Tooltip>
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=move |_| delete_note(signal.id)
                            >
                                "Delete"
                            </Button>
                        </Space>
                    }
                }
            />
        </Card>
    }
}
