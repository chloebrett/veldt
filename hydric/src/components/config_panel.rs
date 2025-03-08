use leptos::prelude::*;

use shared::model::{Scale, ScaleValue};
use shared::types::Beats;
use strum::IntoEnumIterator;
use thaw::{Card, Select, Slider, Space, SpinButton};

#[component]
pub fn ConfigPanel(
    wave: RwSignal<String>,
    bpm: RwSignal<Beats>,
    volume: RwSignal<f64>,
    key: RwSignal<String>,
    scale: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Card>
            <Select value=wave>
                <option>Sine</option>
                <option>Square</option>
                <option>Saw</option>
                <option>Triangle</option>
            </Select>
            <Space>
                <p>BPM</p>
                <SpinButton<f32> step_page=1.0 min=20.0 max=400.0 value=bpm />
                <p>Volume</p>
                <Slider value=volume />
                <Select value=key>
                    {ScaleValue::iter()
                        .map(|scale_value| view! { <option>{scale_value.to_string()}</option> })
                        .collect_view()}
                </Select>
                <Select value=scale>
                    {Scale::iter()
                        .map(|scale_name| view! { <option>{scale_name.to_string()}</option> })
                        .collect_view()}
                </Select>
            </Space>
        </Card>
    }
}
