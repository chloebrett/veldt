use crate::player::AudioPlayer;
use crate::render::render;
use leptos::prelude::*;
use mesic::{create_track, render as local_render};
use shared::model::note::Note;
use shared::model::wave_type::WaveType;
use shared::types::Beats;
use std::str::FromStr;
use thaw::{
    Accordion, AccordionHeader, AccordionItem, Button, ButtonAppearance, Card, ConfigProvider,
    Select, Slider, Space, SpinButton,
};

#[component]
pub fn App() -> impl IntoView {
    let (_player, set_player) = signal_local(None::<AudioPlayer>);
    let wave_string = RwSignal::new(WaveType::Sine.to_string());
    let bpm_value = RwSignal::<Beats>::new(120.0);
    let volume_percent = RwSignal::new(100.0f64);
    let transpose_semitones = RwSignal::new(0);

    let initial_notes = vec![
        (0, ArcRwSignal::new(0.0), ArcRwSignal::new(1.0)),
        (1, ArcRwSignal::new(1.0), ArcRwSignal::new(1.0)),
        (2, ArcRwSignal::new(2.0), ArcRwSignal::new(2.0)),
    ];
    let next_note_id = RwSignal::new(initial_notes.len());
    let (notes, set_notes) = signal_local(initial_notes);

    let add_note = move |_| {
        let note = (
            next_note_id.get(),
            // Arc so that they get cleaned up when removed.
            ArcRwSignal::new(0.0),
            ArcRwSignal::new(1.0),
        );

        set_notes.update(move |notes| notes.push(note));

        next_note_id.update(|it| *it += 1);
    };

    let delete_note = move |id| {
        set_notes.update(move |notes| notes.retain(|note| note.0 != id));
    };

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();
    let bpm = move || bpm_value.get();
    let volume = move || (volume_percent.get() / 100.0f64) as f32;

    let track = move || {
        create_track(
            notes
                .get()
                .into_iter()
                .map(|(_, pitch, duration)| Note(pitch.get(), duration.get()))
                .collect(),
            wave(),
            bpm(),
            volume(),
            transpose_semitones.get(),
        )
    };

    // Continually re-request audio from the server then the wave type changes.
    let server_audio = LocalResource::new(move || render(track()));

    view! {
        <ConfigProvider>
            <Accordion collapsible=true>
                <AccordionItem value="egui">
                    <AccordionHeader slot>egui canvas</AccordionHeader>
                    <div id="egui_canvas_parent">
                        <canvas id="egui_canvas"></canvas>
                    </div>
                </AccordionItem>
            </Accordion>
            <h1>"Veldt"</h1>
            <Card>
                <Space>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=move |_| {
                            set_player
                                .set(AudioPlayer::new(&local_render(&track())).unwrap().into());
                        }
                    >
                        "Play (rendered in browser)"
                    </Button>
                    <Suspense fallback=move || {
                        view! {
                            <Button disabled=true appearance=ButtonAppearance::Secondary>
                                "Play (rendered on server)"
                            </Button>
                        }
                    }>
                        {move || {
                            server_audio
                                .get()
                                .map(move |audio| {
                                    view! {
                                        <Button
                                            appearance=ButtonAppearance::Secondary
                                            on_click=move |_| {
                                                let audio = audio.clone().take();
                                                set_player.set(AudioPlayer::new(&audio).unwrap().into());
                                            }
                                        >
                                            "Play (rendered on server)"
                                        </Button>
                                    }
                                })
                        }}
                    </Suspense>
                    <Select value=wave_string>
                        <option>Sine</option>
                        <option>Square</option>
                        <option>Saw</option>
                        <option>Triangle</option>
                    </Select>
                </Space>
            </Card>
            <Card>
                <Space>
                    <p>BPM</p>
                    <SpinButton<f32> step_page=1.0 min=20.0 max=400.0 value=bpm_value />
                    <p>Volume</p>
                    <Slider value=volume_percent />
                    <p>Transpose</p>
                    <SpinButton<i32> value=transpose_semitones step_page=1 min=-24 max=24 />
                </Space>
            </Card>
            <Card>
                <Button appearance=ButtonAppearance::Secondary on_click=add_note>
                    "Add note"
                </Button>
                <For
                    each=move || notes.get()
                    key=|note| note.0
                    children=move |(id, pitch, duration)| {
                        let pitch = RwSignal::from(pitch);
                        let duration = RwSignal::from(duration);

                        view! {
                            <Space>
                                <SpinButton<f32> value=pitch step_page=1.0 min=-24.0 max=24.0 />
                                <SpinButton<f32> value=duration step_page=0.25 min=0.5 max=16.0 />
                                <Button appearance=ButtonAppearance::Secondary on_click=move |_| delete_note(id)>"Delete"</Button>
                            </Space>
                        }
                    }
                />
            </Card>
        </ConfigProvider>
    }
}
