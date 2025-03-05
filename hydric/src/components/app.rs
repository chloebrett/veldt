use crate::components::config_panel::ConfigPanel;
use crate::components::envelope_panel::EnvelopePanel;
use crate::components::notes_panel::{NoteSignal, NotesPanel};
use crate::components::playback::Playback;
use leptos::prelude::*;
use mesic::create_track;
use shared::model::adsr_envelope::AdsrEnvelope;
use shared::model::scale::Scale;
use shared::model::scale_value::ScaleValue;
use shared::model::wave_type::WaveType;
use shared::serialize::map_vec;
use shared::types::{Beats, PitchValue};
use std::str::FromStr;
use thaw::{Accordion, AccordionHeader, AccordionItem, ConfigProvider};

#[component]
pub fn App() -> impl IntoView {
    let notes = RwSignal::<Vec<NoteSignal>>::new(vec![]);

    let wave_string = RwSignal::new(WaveType::Sine.to_string());
    let bpm_value = RwSignal::<Beats>::new(120.0);
    let volume_percent = RwSignal::new(100.0f64);
    let transpose_interval = RwSignal::<PitchValue>::new(0);
    let key_string = RwSignal::new(ScaleValue::A.to_string());
    let scale_string = RwSignal::new(Scale::Chromatic.to_string());

    let attack = RwSignal::new(0.1);
    let decay = RwSignal::new(0.1);
    let sustain = RwSignal::new(0.8);
    let release = RwSignal::new(0.2);

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();
    let bpm = move || bpm_value.get();
    let volume = move || (volume_percent.get() / 100.0f64) as f32;
    let envelope = move || AdsrEnvelope {
        attack: attack.get(),
        decay: decay.get(),
        sustain: sustain.get(),
        release: release.get(),
    };

    let track = Memo::new(move |_| {
        create_track(
            map_vec(notes.get()),
            wave(),
            bpm(),
            volume(),
            transpose_interval.get(),
            envelope(),
        )
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
            <Playback track=track />
            <ConfigPanel
                wave=wave_string
                bpm=bpm_value
                volume=volume_percent
                transpose=transpose_interval
                key=key_string
                scale=scale_string
            />
            <EnvelopePanel attack=attack decay=decay sustain=sustain release=release />
            <NotesPanel notes=notes scale_string=scale_string key_string=key_string/>
        </ConfigProvider>
    }
}
