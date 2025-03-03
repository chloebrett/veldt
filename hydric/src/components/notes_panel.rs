use leptos::prelude::*;
use std::str::FromStr;
use shared::model::pitch_name::PitchName;
use shared::model::scale_value::ScaleValue;
use shared::model::note::Note;
use shared::types::{Beats, Octave};
use thaw::{Button, ButtonAppearance, Card, Select, Space, SpinButton};

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
                octave: item.octave.get()
            },
            beats: item.duration.get()
        }
    }
}

#[component]
pub fn NotesPanel(notes: RwSignal<Vec<NoteSignal>>) -> impl IntoView {
    let next_note_id = RwSignal::new(notes.get_untracked().len() as u32);

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
                            <Select value=scale_value>
                                <option>{ScaleValue::A.to_string()}</option>
                                <option>{ScaleValue::ASharp.to_string()}</option>
                                <option>{ScaleValue::B.to_string()}</option>
                                <option>{ScaleValue::C.to_string()}</option>
                                <option>{ScaleValue::CSharp.to_string()}</option>
                                <option>{ScaleValue::D.to_string()}</option>
                                <option>{ScaleValue::DSharp.to_string()}</option>
                                <option>{ScaleValue::E.to_string()}</option>
                                <option>{ScaleValue::F.to_string()}</option>
                                <option>{ScaleValue::FSharp.to_string()}</option>
                                <option>{ScaleValue::G.to_string()}</option>
                                <option>{ScaleValue::GSharp.to_string()}</option>
                            </Select>
                            <SpinButton<i32> value=octave step_page=1 min=0 max=12 />
                            <SpinButton<f32> value=duration step_page=0.25 min=0.5 max=16.0 />
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
