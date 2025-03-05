use leptos::prelude::*;

use thaw::{Card, Space, SpinButton, Tooltip};

#[component]
pub fn DetunePanel(osc_count: RwSignal<u32>, detune_cents: RwSignal<f32>) -> impl IntoView {
    view! {
        <Card>
            <Space>
                <Tooltip content="Number of oscillators">
                    <SpinButton<u32> step_page=1 min=1 max=32 value=osc_count />
                </Tooltip>
                <Tooltip content="Detune (cents)">
                    <SpinButton<f32> step_page=1.0 min=0.0 max=100.0 value=detune_cents />
                </Tooltip>
            </Space>
        </Card>
    }
}
