use crate::{audio_player::AudioPlayer, note_save::load_note_list};
use crate::audio_render::render as server_render;
use leptos::prelude::*;
use mesic::{SupersawConfig, render as local_render};
use shared::model::track::Track;
use thaw::{Button, ButtonAppearance, Card, Select, Space};

#[component]
pub fn Playback(
    track: Memo<Track>,
    supersaw_config: Memo<SupersawConfig>,
    resonant_freq: RwSignal<f32>,
    resonance_q: RwSignal<f32>,
    resonance_wet: RwSignal<f32>,
) -> impl IntoView {
    let (_, set_player) = signal_local(None::<AudioPlayer>);

    // Continually re-request audio from the server then the wave type changes.
    let server_audio = LocalResource::new(move || server_render(track.get().clone()));

    let saved_notes_name = RwSignal::new(String::from("My Song")); 
    
    let load_saved_names = LocalResource::new(move || load_note_list());
    let load = move || {
        load_saved_names
            .get()
            .map(|name| name.take())
            .unwrap_or_else(|| vec!(saved_notes_name.get()))
    };

    view! {
        <Card>
            <Space>
                <Button
                    appearance=ButtonAppearance::Primary
                    on_click=move |_| {
                        set_player
                            .set(
                                AudioPlayer::new(
                                        &local_render(
                                            &track.get(),
                                            supersaw_config.get(),
                                            resonant_freq.get(),
                                            resonance_q.get(),
                                            resonance_wet.get(),
                                        ),
                                    )
                                    .unwrap()
                                    .into(),
                            );
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
                <Suspense fallback=move || {
                    view! {
                        <Select disabled=true value=saved_notes_name>
                            <option>{saved_notes_name.get()}</option>
                        </Select>
                    }
                }>
                    <Select value=saved_notes_name>
                        <For
                            each=load
                            key=|name| name.clone()
                            children= move |name| {
                                view! {
                                    <option>{name}</option>
                                }
                            }
                        />
                    </Select>
                </Suspense>
            </Space>
        </Card>
    }
}
