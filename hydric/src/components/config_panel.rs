use leptos::prelude::*;

use shared::types::{Beats, PitchValue};
use shared::model::scale_value::ScaleValue;
use thaw::{Card, Select, Slider, Space, SpinButton};
use strum::IntoEnumIterator;

#[component]
pub fn ConfigPanel(
    wave: RwSignal<String>,
    bpm: RwSignal<Beats>,
    volume: RwSignal<f64>,
    transpose: RwSignal<PitchValue>,
    key: RwSignal<String>,
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
                <SpinButton<PitchValue> value=transpose step_page=1 min=-24 max=24 />
                <Select value=key>
                    {
                        ScaleValue::iter()
                            .map(|scale_value| view! {<option>{scale_value.to_string()}</option>})
                            .collect_view()
                    }
                </Select>
            </Space>
        </Card>
    }
}
