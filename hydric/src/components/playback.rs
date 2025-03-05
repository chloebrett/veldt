use crate::audio_player::AudioPlayer;
use crate::audio_render::render as server_render;
use leptos::prelude::*;
use mesic::{SupersawConfig, render as local_render};
use shared::model::track::Track;
use thaw::{Button, ButtonAppearance, Card, Space};

#[component]
pub fn Playback(track: Memo<Track>, supersaw_config: Memo<SupersawConfig>) -> impl IntoView {
    let (_, set_player) = signal_local(None::<AudioPlayer>);

    // Continually re-request audio from the server then the wave type changes.
    let server_audio = LocalResource::new(move || server_render(track.get().clone()));

    view! {
        <Card>
            <Space>
                <Button
                    appearance=ButtonAppearance::Primary
                    on_click=move |_| {
                        set_player
                            .set(
                                AudioPlayer::new(&local_render(&track.get(), supersaw_config.get()))
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
            </Space>
        </Card>
    }
}
