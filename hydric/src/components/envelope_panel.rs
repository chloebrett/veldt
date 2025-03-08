use crate::state::subfield;
use leptos::prelude::*;
use shared::model::AdsrEnvelope;

use thaw::{Card, Space, SpinButton, Tooltip};

#[derive(Clone)]
pub struct TrackedAdsrEnvelope {
    pub attack: RwSignal<f32>,
    pub decay: RwSignal<f32>,
    pub sustain: RwSignal<f32>,
    pub release: RwSignal<f32>,
}

impl From<TrackedAdsrEnvelope> for AdsrEnvelope {
    fn from(val: TrackedAdsrEnvelope) -> AdsrEnvelope {
        AdsrEnvelope {
            attack: val.attack.get(),
            decay: val.decay.get(),
            sustain: val.sustain.get(),
            release: val.release.get(),
        }
    }
}

#[component]
pub fn EnvelopePanel(envelope: RwSignal<TrackedAdsrEnvelope, LocalStorage>) -> impl IntoView {
    let e = envelope;
    let attack = subfield(&e, |e| e.attack, move |a| e.get().attack.set(a));
    let decay = subfield(&e, |e| e.decay, move |d| e.get().decay.set(d));
    let sustain = subfield(&e, |e| e.sustain, move |s| e.get().sustain.set(s));
    let release = subfield(&e, |e| e.release, move |r| e.get().release.set(r));

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
