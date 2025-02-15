use leptos::*;
use leptos::prelude::*;
use thaw::{Card, Button, Space, ConfigProvider, ButtonAppearance, Select};
use shared::model::wave_type::WaveType;
use shared::model::demo_option::DemoOption;
use std::str::FromStr;

use crate::ping::ping;
use crate::render::render;
use crate::player::AudioPlayer;

#[component]
pub fn App() -> impl IntoView {
    let (_player, set_player) = signal_local(None::<AudioPlayer>);
    let wave_string = RwSignal::new(WaveType::Sine.to_string());
    let demo_string = RwSignal::new(DemoOption::Demo1.to_string());

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();
    let demo_option = move || DemoOption::from_str(&demo_string.get()).unwrap();

    let ping_action = Action::new_local(|_: &()| async {
        ping().await;
    });

    // Continually re-request audio from the server then the wave type changes.
    let server_audio = LocalResource::new(move || {
        render(demo_option(), wave())
    });

    view! {
        <ConfigProvider>
            <div id="egui_canvas_parent"><canvas id="egui_canvas"></canvas></div>
            <h1>"Veldt"</h1>
            <Card>
                <Space>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=move |_| {
                            set_player
                                .set(AudioPlayer::new(&mesic::demo_floats(demo_option(), wave())).unwrap().into());
                        }
                    >
                        "Play (rendered in browser)"
                    </Button>
                    <Button
                        appearance=ButtonAppearance::Secondary
                        on_click=move |_| {
                            ping_action.dispatch(());
                        }
                    >
                        "Ping server (check network tab)"
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
                    <Select value=demo_string>
                        <option>Demo1</option>
                        <option>Demo2</option>
                    </Select>
                </Space>
            </Card>
        </ConfigProvider>
    }
}

