use leptos::prelude::*;

use shared::model::note::Note;
use shared::types::{Beats, Semitones};
use thaw::{Button, ButtonAppearance, Card, Space, SpinButton};

#[derive(Clone)]
pub struct NoteSignal {
    id: u32,
    // Arc so that they get cleaned up when removed from the notes list.
    pub pitch: ArcRwSignal<Semitones>,
    pub duration: ArcRwSignal<Beats>,
}

impl From<NoteSignal> for Note {
    fn from(item: NoteSignal) -> Note {
        Note(item.pitch.get(), item.duration.get())
    }
}

#[component]
pub fn NotesPanel(notes: RwSignal<Vec<NoteSignal>>) -> impl IntoView {
    let next_note_id = RwSignal::new(notes.get().len() as u32);

    let add_note = move |_| {
        let note = NoteSignal {
            id: next_note_id.get(),
            pitch: ArcRwSignal::new(0.0),
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
                    let pitch = RwSignal::from(signal.pitch);
                    let duration = RwSignal::from(signal.duration);

                    view! {
                        <Space>
                            <SpinButton<f32> value=pitch step_page=1.0 min=-24.0 max=24.0 />
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
