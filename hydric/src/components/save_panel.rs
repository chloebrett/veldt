use shared::serialize::map_vec;
use shared::model::note::Note;
use leptos::prelude::*;
use crate::components::notes_panel::NoteSignal;
use crate::note_save::save_notes;
use thaw::{Button, FieldContextProvider, ButtonAppearance, Input, Field, Card, Space};

#[component]
pub fn SavePanel (
    notes: RwSignal<Vec<NoteSignal>>,
    ) -> impl IntoView {
    let save_string = RwSignal::new(String::from("My Song"));
    let save = Action::new_local(|input: &(String, Vec<Note>)| {
        let input = input.to_owned();
        async move {
            save_notes(
                input.0,
                input.1,
            ).await
        }
    });
    view! {
        <Card>
            <Field label="Save Name">
                <Input value=save_string/>
            </Field>
            <Button
                appearance=ButtonAppearance::Secondary
                on_click= move |_| {
                    let save_name = save_string.get();
                    let save_notes = map_vec::<NoteSignal, Note>(notes.get());
                    save.dispatch((save_name, save_notes));
                }
            >
                "Save"
            </Button>
        </Card>
    }
} 
