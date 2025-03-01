use leptos::prelude::*;
use shared::model::demo_option::DemoOption;
use shared::model::wave_type::WaveType;
use std::str::FromStr;
use thaw::{
    Accordion, AccordionHeader, AccordionItem, Button, ButtonAppearance, Card, ConfigProvider,
    Select, Slider, Space, SpinButton,
};
use mesic::create_track;

use crate::ping::ping;
use crate::player::AudioPlayer;
use crate::render::render;

#[component]
pub fn App() -> impl IntoView {
    let (_player, set_player) = signal_local(None::<AudioPlayer>);
    let wave_string = RwSignal::new(WaveType::Sine.to_string());
    let demo_string = RwSignal::new(DemoOption::Overworld.to_string());
    let volume_percent = RwSignal::new(100.0f64);
    let transpose_semitones = RwSignal::new(0);

    // Hard coded as three for now; will be generalized into N soon
    let note1 = RwSignal::new(0);
    let note2 = RwSignal::new(1);
    let note3 = RwSignal::new(2);

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();
    let demo_option = move || DemoOption::from_str(&demo_string.get()).unwrap();
    let volume = move || (volume_percent.get() / 100.0f64) as f32;
    let notes = move || vec!(note1.get(), note2.get(), note3.get());
    let custom_demo = move || create_track(notes(), wave(), volume(), transpose_semitones.get());

    let ping_action = Action::new_local(|_: &()| async {
        ping().await;
    });

    // Continually re-request audio from the server then the wave type changes.
    let server_audio =
        LocalResource::new(move || render(demo_option(), custom_demo(), wave(), volume(), transpose_semitones.get()));

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
                                .set(
                                    AudioPlayer::new(
                                            &mesic::demo_floats(demo_option(), wave(), volume(), transpose_semitones.get()),
                                        )
                                        .unwrap()
                                        .into(),
                                );
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
                        <option>Overworld</option>
                        <option>FurElise</option>
                        <option>Custom</option>
                    </Select>
                    <Slider value=volume_percent />
                </Space>
            </Card>
            <Card>
                    <SpinButton<i32> value=transpose_semitones step_page=1 min=-24 max=24 />
            </Card>
            <Card>
                    <SpinButton<i32> value=note1 step_page=1 min=-24 max=24 />
                    <SpinButton<i32> value=note2 step_page=1 min=-24 max=24 />
                    <SpinButton<i32> value=note3 step_page=1 min=-24 max=24 />
            </Card>
        </ConfigProvider>
    }
}
