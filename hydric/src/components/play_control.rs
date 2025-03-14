use crate::audio_player::{Handle, play};
use super::envelope_control;
use crate::rpc::render;
use egui::{
    Color32, Rect, ScrollArea, Ui, containers::Frame, emath, epaint, epaint::PathStroke, pos2,
    scroll_area::ScrollBarVisibility, vec2,
};
use mesic::{SAMPLE_RATE, create_scale_values, create_track, render as local_render};
use poll_promise::Promise;
use shared::model::{
    AdsrEnvelope, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig, EqType,
    GeneratorInstance, GeneratorMeta, GeneratorType, Note, PitchName, Scale, ScaleValue,
    SimpleWaveConfig, WaveType,
};
use strum::IntoEnumIterator;
use super::audio_vis::audio_vis;
use super::app::App;

pub fn play_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play (local)").clicked() {
        let track = create_track(app.notes.clone());
        let delay = EffectInstance {
            effect: Effect::SimpleDelay {
                config: DelayConfig {
                    amplitude: app.delay_amplitude,

                    delay_ms: app.delay_ms,
                },
            },
            meta: EffectMeta {
                id: 0,
                wet: app.delay_wet,
            },
        };
        let eq = EffectInstance {
            effect: Effect::SimpleEq {
                config: EqConfig {
                    kind: app.eq_type.clone(),
                    fc: app.resonant_freq,
                    q: app.q,
                },
            },
            meta: EffectMeta {
                id: 1,
                wet: app.eq_wet,
            },
        };
        let effects = vec![delay, eq];
        app.audio = local_render(&track, effects, app.generator.clone(), app.bpm)
            .into_iter()
            .map(|sample| sample.clamp(-1.0, 1.0))
            .collect();
        app.handle = Some(play(&app.audio));
    }
    if let Some(render_promise) = &app.server_render_promise {
        if let Some(Some(server_audio)) = render_promise.ready() {
            if ui.button("Play (server)").clicked() {
                app.audio = server_audio
                    .to_vec()
                    .iter()
                    .copied()
                    .map(|sample| sample.clamp(-1.0, 1.0))
                    .collect::<Vec<f32>>();
                app.handle = Some(play(&app.audio))
            }
        }
    }
    if ui.button("Load audio (server)").clicked() {
        let track = create_track(app.notes.clone());
        let delay = EffectInstance {
            effect: Effect::SimpleDelay {
                config: DelayConfig {
                    amplitude: app.delay_amplitude,

                    delay_ms: app.delay_ms,
                },
            },
            meta: EffectMeta {
                id: 0,
                wet: app.delay_wet,
            },
        };
        let eq = EffectInstance {
            effect: Effect::SimpleEq {
                config: EqConfig {
                    kind: app.eq_type.clone(),
                    fc: app.resonant_freq,
                    q: app.q,
                },
            },
            meta: EffectMeta {
                id: 1,
                wet: app.eq_wet,
            },
        };
        let effects = vec![delay, eq];
        let generator = app.generator.clone();
        let bpm = app.bpm;
        app.server_render_promise = Some(Promise::spawn_local(async move {
            render(track, effects, generator, bpm).await
        }))
    }
    audio_vis(app, ui);
}
