use super::config_panel::ConfigPanel;
use super::detune_panel::DetunePanel;
use super::envelope_panel::{EnvelopePanel, TrackedAdsrEnvelope};
use super::notes_panel::{NoteSignal, NotesPanel};
use super::playback::Playback;
use super::resonance_panel::ResonancePanel;
use super::save_panel::SavePanel;
use leptos::prelude::*;
use mesic::{SupersawConfig, create_track};
use shared::model::{Scale, ScaleValue, WaveType};
use shared::serialize::map_vec;
use shared::types::Beats;
use std::str::FromStr;
use thaw::ConfigProvider;

#[component]
pub fn App() -> impl IntoView {
    let notes = RwSignal::<Vec<NoteSignal>>::new(vec![]);
    let saved_notes_names = RwSignal::<Vec<String>>::new(vec![String::from("My Song")]);

    let wave_string = RwSignal::new(WaveType::Sine.to_string());
    let bpm_value = RwSignal::<Beats>::new(120.0);
    let volume_percent = RwSignal::new(100.0f64);
    let key_string = RwSignal::new(ScaleValue::A.to_string());
    let scale_string = RwSignal::new(Scale::Chromatic.to_string());

    let osc_count = RwSignal::new(4u32);
    let detune_cents = RwSignal::new(5.0);

    let resonant_freq = RwSignal::new(2000.0);
    let resonance_q = RwSignal::new(1.0);
    let resonance_wet = RwSignal::new(1.0);

    // TODO: instead of using a dependent signal, consider implementing
    // the appropriate From trait.
    let wave = move || WaveType::from_str(&wave_string.get()).unwrap();
    let bpm = move || bpm_value.get();
    let volume = move || (volume_percent.get() / 100.0f64) as f32;
    let envelope = RwSignal::new_local(TrackedAdsrEnvelope {
        attack: RwSignal::new(0.1),
        decay: RwSignal::new(0.1),
        sustain: RwSignal::new(0.8),
        release: RwSignal::new(0.2),
    });
    // TODO: put the supersaw config in Track, etc.
    // Really this is a generator config.
    let supersaw_config = Memo::new(move |_| SupersawConfig {
        osc_count: osc_count.get(),
        detune_cents: detune_cents.get(),
    });

    let track = Memo::new(move |_| {
        create_track(
            map_vec(notes.get()),
            wave(),
            bpm(),
            volume(),
            envelope.get().into(),
        )
    });

    view! {
        <ConfigProvider>
            <div id="egui_canvas_parent">
                <canvas id="egui_canvas"></canvas>
            </div>
            <h1>"Veldt"</h1>
            <Playback
                track=track
                saved_notes_names=saved_notes_names
                supersaw_config=supersaw_config
                resonant_freq=resonant_freq
                resonance_q=resonance_q
                resonance_wet=resonance_wet
            />
            <ConfigPanel
                wave=wave_string
                bpm=bpm_value
                volume=volume_percent
                key=key_string
                scale=scale_string
            />
            <EnvelopePanel envelope=envelope />
            <DetunePanel osc_count=osc_count detune_cents=detune_cents />
            <ResonancePanel
                resonant_freq=resonant_freq
                resonance_q=resonance_q
                resonance_wet=resonance_wet
            />
            <NotesPanel notes=notes scale_string=scale_string key_string=key_string />
            <SavePanel notes=notes saved_note_names=saved_notes_names />
        </ConfigProvider>
    }
}
