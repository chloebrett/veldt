use crate::audio_player::{Handle, play};
use crate::audio_render::render;
use crate::envelope_control;
use crate::note_save::{load_note_list, load_notes, save_notes};
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

fn play_control(app: &mut App, ui: &mut Ui) {
    if ui.button("Play (local)").clicked() {
        let track = create_track(self.notes.clone());
        let delay = EffectInstance {
            effect: Effect::SimpleDelay {
                config: DelayConfig {
                    amplitude: self.delay_amplitude,

                    delay_ms: self.delay_ms,
                },
            },
            meta: EffectMeta {
                id: 0,
                wet: self.delay_wet,
            },
        };
        let eq = EffectInstance {
            effect: Effect::SimpleEq {
                config: EqConfig {
                    kind: self.eq_type.clone(),
                    fc: self.resonant_freq,
                    q: self.q,
                },
            },
            meta: EffectMeta {
                id: 1,
                wet: self.eq_wet,
            },
        };
        let effects = vec![delay, eq];
        self.audio = local_render(&track, effects, self.generator.clone(), self.bpm)
            .into_iter()
            .map(|sample| sample.clamp(-1.0, 1.0))
            .collect();
        self.handle = Some(play(&self.audio));
    }
    if let Some(render_promise) = &self.server_render_promise {
        if let Some(Some(server_audio)) = render_promise.ready() {
            if ui.button("Play (server)").clicked() {
                self.audio = server_audio
                    .to_vec()
                    .iter()
                    .copied()
                    .map(|sample| sample.clamp(-1.0, 1.0))
                    .collect::<Vec<f32>>();
                self.handle = Some(play(&self.audio))
            }
        }
    }
    if ui.button("Load audio (server)").clicked() {
        let track = create_track(self.notes.clone());
        let delay = EffectInstance {
            effect: Effect::SimpleDelay {
                config: DelayConfig {
                    amplitude: self.delay_amplitude,

                    delay_ms: self.delay_ms,
                },
            },
            meta: EffectMeta {
                id: 0,
                wet: self.delay_wet,
            },
        };
        let eq = EffectInstance {
            effect: Effect::SimpleEq {
                config: EqConfig {
                    kind: self.eq_type.clone(),
                    fc: self.resonant_freq,
                    q: self.q,
                },
            },
            meta: EffectMeta {
                id: 1,
                wet: self.eq_wet,
            },
        };
        let effects = vec![delay, eq];
        let generator = self.generator.clone();
        let bpm = self.bpm;
        self.server_render_promise = Some(Promise::spawn_local(async move {
            render(track, effects, generator, bpm).await
        }))
    }
    self.audio_vis(ui);
}
