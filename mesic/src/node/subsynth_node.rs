use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::consts::CHANNEL_COUNT;
use crate::envelope::EnvelopeGenerator;
use crate::graph::{NoteEventType, ProcessContext};
use crate::wave::{WaveCache, WaveKey, detune_multiplier, linspace};
use dasp_frame::Stereo;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{
    AntiAliasingMode, Generator, GeneratorInstance, GeneratorMeta, Oscillator, PitchName,
    SubSynthConfig,
};
use shared::types::{Freq, KnobPosition, Volume};
use state::GeneratorSelector;

pub struct SubSynthNode {
    selector: GeneratorSelector,
    state: NodeState,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    config: SubSynthConfig,
    meta: GeneratorMeta,
    voice: Voice,
}

struct Voice {
    egs: [EnvelopeGenerator; 3],
    sources: Option<[SubSynthWaveSource; 3]>,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = SubSynthConfig::default();
        let egs: Vec<_> = config
            .envelopes
            .iter()
            .map(|env| EnvelopeGenerator::new(env.clone()))
            .collect();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            voice: Voice {
                egs: [egs[0].clone(), egs[1].clone(), egs[2].clone()],
                sources: None,
            },
        }
    }
}

impl NodeState {
    // TODO: update logic is almost the same as the simple wave generator.
    // Should it be de-duplicated?
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        if let GeneratorInstance {
            it: Generator::SubSynth(config),
            meta,
            ..
        } = &payload.store.select(&selector)
        {
            if self.config != *config {
                self.config = config.clone();
            }
            if self.meta != *meta {
                self.meta = meta.clone();
            }
        }
    }
}

impl SubSynthNode {
    pub fn new(selector: GeneratorSelector) -> Self {
        Self {
            selector,
            state: NodeState::default(),
        }
    }

    // TODO: this logic is similar and shared with subsynth and simple wave, probably should move
    fn apply_volume_and_pan(
        buffer: &mut Buffer,
        channel_index: usize,
        volume: Volume,
        pan: KnobPosition,
    ) {
        let pan_mult = pan_multipliers(pan)[channel_index];
        for x in buffer.iter_mut() {
            *x *= pan_mult * volume;
        }
    }
}

impl Node<ProcessContext> for SubSynthNode {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let state = &mut self.state;
        state.update(payload, self.selector);

        // Skip generating if muted!
        // TODO: disconnect muted generators from the graph.
        // This should be handled from the mixer.
        if state.meta.mute || state.meta.volume == 0.0 {
            return;
        }

        let mut buffer = Buffer::SILENT;
        let GeneratorSelector(generator_index) = self.selector;

        // TODO: fix this, it's n^2 right now. (well, n*64).
        for i in 0..buffer.len() {
            let mut events: Vec<_> = payload.note_events[generator_index]
                .clone()
                .into_iter()
                .filter(|it| it.sample_index == i)
                .collect();

            // Special case: if there are both note_on and note_off events in a single sample,
            // don't process the note_off events.
            if events.iter().any(|it| it.kind == NoteEventType::On) {
                events.retain(|it| it.kind == NoteEventType::On);
            }

            for note_event in events {
                match &note_event.kind {
                    NoteEventType::On => {
                        log::info!(
                            "Note on event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        let mut sources = vec![];
                        for i in 0..state.config.envelopes.len() {
                            let eg = &mut state.voice.egs[i];
                            let env = state.config.envelopes[i].clone();
                            let osc = state.config.oscillators[i].clone();

                            eg.note_on();
                            eg.set_envelope(env);
                            // TODO: update config dynamically, not just when starting a new note.
                            sources.push(SubSynthWaveSource::new(note_event.pitch_name, osc));
                        }
                        state.voice.sources = Some(sources.try_into().unwrap());
                    }
                    NoteEventType::Off => {
                        log::info!(
                            "Note off event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        // TODO: check against start/stop time too?
                        if let Some(sources) = &state.voice.sources {
                            for (eg, source) in state.voice.egs.iter_mut().zip(sources.iter()) {
                                if source.same_pitch(note_event.pitch_name) {
                                    eg.note_off();
                                }
                            }
                        }
                    }
                }
            }

            if let Some(sources) = &mut state.voice.sources {
                for (eg, source) in state.voice.egs.iter_mut().zip(sources.iter_mut()) {
                    let amp = eg.next().unwrap_or(0.0);
                    let wave = source.next().unwrap_or([0.0; 2]);

                    // TODO: stereo
                    buffer[i] += amp * wave[0];
                }
            }
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            let meta = &self.state.meta;
            Self::apply_volume_and_pan(out_buf, channel_index, meta.volume, meta.pan);
        }
    }
}

#[derive(Debug)]
pub struct SubSynthWaveSource {
    // TODO: recycle the wave cache?
    // Currently it's re-created each time the note changes.
    cache: WaveCache,
    pitch: PitchName,
    oscillator: Oscillator,
    sample_index: usize,
}

impl SubSynthWaveSource {
    pub fn new(pitch: PitchName, oscillator: Oscillator) -> Self {
        Self {
            cache: WaveCache::default(),
            pitch,
            oscillator,
            sample_index: 0,
        }
    }

    pub fn same_pitch(&self, pitch: PitchName) -> bool {
        self.pitch == pitch
    }
}

impl Iterator for SubSynthWaveSource {
    type Item = Stereo<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut output_mono = 0.0;
        let freq: Freq = self.pitch.into();
        let osc = &self.oscillator;
        let freq = freq * detune_multiplier(osc.osc_detune);

        let detunes = linspace(-osc.unison_detune, osc.unison_detune, osc.osc_count);

        for detune in detunes {
            let freq = freq * detune_multiplier(detune);
            let step = freq / (SAMPLE_RATE as f32);
            let phase = ((self.sample_index as f32) * step) % 1.0;

            let key = WaveKey {
                kind: osc.wave,
                aa: AntiAliasingMode::Off,
                freq: freq.into(),
            };

            if osc.osc_detune != 0.0 {
                output_mono += self.cache.get(&key, phase) / (osc.osc_detune as f32).sqrt();
            } else {
                output_mono += self.cache.get(&key, phase) / osc.osc_detune as f32;
            }
        }

        let mut output_stereo = [output_mono; CHANNEL_COUNT];
        for (channel_index, out) in output_stereo.iter_mut().enumerate() {
            let pan_mult = pan_multipliers(osc.pan)[channel_index];
            *out *= pan_mult * osc.volume;
        }

        self.sample_index += 1;
        Some(output_stereo)
    }
}
