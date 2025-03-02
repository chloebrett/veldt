use leptos::prelude::*;
use shared::model::demo_option::DemoOption;
use shared::model::wave_type::WaveType;
use shared::types::Beats;
use std::str::FromStr;
use thaw::{
    Accordion, AccordionHeader, AccordionItem, Button, ButtonAppearance, Card, ConfigProvider,
    Select, Slider, Space, SpinButton,
};
use crate::player::AudioPlayer;
use crate::render::render;
use mesic::{create_demo_track, create_track, render as local_render};
use shared::model::note::Note;

#[component]
pub fn App() -> impl IntoView {
    let (_player, set_player) = signal_local(None::<AudioPlayer>);
    let wave_string = RwSignal::new(WaveType::Sine.to_string());
    let demo_string = RwSignal::new(DemoOption::Overworld.to_string());
    let bpm_value = RwSignal::<Beats>::new(120.0);
    let volume_percent = RwSignal::new(100.0f64);
    let transpose_semitones = RwSignal::new(0);

    // Hard coded as three for now; will be generalized into N soon
    let note1 = RwSignal::new(0);
    let note2 = RwSignal::new(1);
    let note3 = RwSignal::new(2);
    let duration1 = RwSignal::new(1.0);
    let duration2 = RwSignal::new(1.0);
    let duration3 = RwSignal::new(2.0);

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();
    let demo_option = move || DemoOption::from_str(&demo_string.get()).unwrap();
    let bpm = move || bpm_value.get();
    let volume = move || (volume_percent.get() / 100.0f64) as f32;
    let notes = move || {
        vec![
            Note(note1.get() as f32, duration1.get()),
            Note(note2.get() as f32, duration2.get()),
            Note(note3.get() as f32, duration3.get()),
        ]
    };

    let track = move || match demo_option() {
        DemoOption::Custom => {
            create_track(notes(), wave(), bpm(), volume(), transpose_semitones.get())
        }
        _ => create_demo_track(
            demo_option(),
            wave(),
            bpm(),
            volume(),
            transpose_semitones.get(),
        ),
    };

    // Continually re-request audio from the server then the wave type changes.
    let server_audio = LocalResource::new(move || {
        render(track())
    });

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
                                            &local_render(&track())
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
                </Space>
            </Card>
            <Card>
                <Space>
                    <SpinButton<f32> step_page=1.0 min=20.0 max=400.0 value=bpm_value/>
                    <Slider value=volume_percent />
                </Space>
            </Card>
            <Card>
                    <SpinButton<i32> value=transpose_semitones step_page=1 min=-24 max=24 />
            </Card>
            <Card>
            <Space>
                    <SpinButton<i32> value=note1 step_page=1 min=-24 max=24 />
                    <SpinButton<f32> value=duration1 step_page=0.25 min=0.5 max=16.0 />
                    </Space>
            <Space>
                    <SpinButton<i32> value=note2 step_page=1 min=-24 max=24 />
                    <SpinButton<f32> value=duration2 step_page=0.25 min=0.5 max=16.0 />
                    </Space>
                    <Space>
                    <SpinButton<i32> value=note3 step_page=1 min=-24 max=24 />
                    <SpinButton<f32> value=duration3 step_page=0.25 min=0.5 max=16.0 />
                    </Space>
            </Card>
        </ConfigProvider>
    }
}
