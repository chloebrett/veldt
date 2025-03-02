use leptos::prelude::*;

use shared::types::Beats;
use thaw::{Card, Select, Slider, Space, SpinButton};

#[component]
pub fn ConfigPanel(
    wave: RwSignal<String>,
    bpm: RwSignal<Beats>,
    volume: RwSignal<f64>,
    transpose: RwSignal<i32>,
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
                <p>Transpose</p>
                <SpinButton<i32> value=transpose step_page=1 min=-24 max=24 />
            </Space>
        </Card>
    }
}
