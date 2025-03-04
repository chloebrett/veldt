use leptos::prelude::*;

use shared::types::{Beats, PitchValue};
use thaw::{Card, Space, SpinButton};

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
                <p>Attack</p>
                <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=attack />
                <p>Decay</p>
                <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=decay />
                <p>Sustain</p>
                <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=sustain />
                <p>Release</p>
                <SpinButton<f32> step_page=0.05 min=0.0 max=1.0 value=release />
            </Space>
        </Card>
    }
}
