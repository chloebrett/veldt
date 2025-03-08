use leptos::prelude::*;

use shared::types::{Freq, KnobPosition};
use thaw::{Card, Space, SpinButton, Tooltip};

#[component]
pub fn ResonancePanel(
    resonant_freq: RwSignal<Freq>,
    resonance_q: RwSignal<KnobPosition>,
    resonance_wet: RwSignal<KnobPosition>,
) -> impl IntoView {
    view! {
        <Card>
            <Space>
                <Tooltip content="Resonant frequency">
                    <SpinButton<f32> step_page=100.0 min=20.0 max=20000.0 value=resonant_freq />
                </Tooltip>
                <Tooltip content="Resonance q-value">
                    <SpinButton<f32> step_page=0.1 min=0.1 max=100.0 value=resonance_q />
                </Tooltip>
                <Tooltip content="Resonance wet">
                    <SpinButton<f32> step_page=0.1 min=0.0 max=1.0 value=resonance_wet />
                </Tooltip>
            </Space>
        </Card>
    }
}
