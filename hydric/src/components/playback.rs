use crate::audio_player::AudioPlayer;
use crate::audio_render::render as server_render;
use leptos::prelude::*;
use mesic::{SupersawConfig, render as local_render};
use shared::model::Track;
use thaw::{Button, ButtonAppearance, Card, Select, Space};
#[component]
pub fn Playback(
    track: Memo<Track>,
    saved_notes_names: RwSignal<Vec<String>>,
    supersaw_config: Memo<SupersawConfig>,
    resonant_freq: RwSignal<f32>,
    resonance_q: RwSignal<f32>,
    resonance_wet: RwSignal<f32>,
) -> impl IntoView {
    let (_, set_player) = signal_local(None::<AudioPlayer>);
    let selected_saved_name = RwSignal::new(String::from("My Song"));

    // Continually re-request audio from the server then the wave type changes.
    let server_audio = LocalResource::new(move || server_render(track.get().clone()));

    let save_names = move || saved_notes_names.get();
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
                <Select value=selected_saved_name>
                    <For
                        each=save_names
                        key=|name| name.clone()
                        children=move |name| {
                            view! { <option>{name}</option> }
                        }
                    />
                </Select>
            </Space>
        </Card>
    }
}
