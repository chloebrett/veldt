use leptos::prelude::*;

use thaw::{Card, Space, SpinButton, Tooltip};

#[component]
pub fn EnvelopePanel(
    attack: RwSignal<f32>,
    decay: RwSignal<f32>,
    sustain: RwSignal<f32>,
    release: RwSignal<f32>,
) -> impl IntoView {
    view! {
        <Card>
            <Space>
                <Tooltip content="Attack">
                    <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=attack />
                </Tooltip>
                <Tooltip content="Decay">
                    <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=decay />
                </Tooltip>
                <Tooltip content="Sustain">
                    <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=sustain />
                </Tooltip>
                <Tooltip content="Release">
                    <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=release />
                </Tooltip>
            </Space>
        </Card>
    }
}
