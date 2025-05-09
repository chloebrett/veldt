use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::envelope::EnvelopeGenerator;
use crate::graph::{NoteEventType, ProcessContext};
use crate::wave::{WaveCache, WaveKey, detune_multiplier, linspace};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Generator, GeneratorInstance, GeneratorMeta, PitchName, SimpleWaveConfig};
use shared::types::Freq;
use state::GeneratorSelector;

pub struct SimpleWaveGeneratorNode {
    selector: GeneratorSelector,
    state: NodeState,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    config: SimpleWaveConfig,
    meta: GeneratorMeta,
    voice: Voice,
}

struct Voice {
    eg: EnvelopeGenerator,
    source: Option<SimpleWaveSource>,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = SimpleWaveConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            voice: Voice {
                eg: EnvelopeGenerator::new(config.envelope.clone()),
                source: None,
            },
        }
    }
}

impl NodeState {
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        if let GeneratorInstance {
            it: Generator::SimpleWave(config),
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

impl SimpleWaveGeneratorNode {
    pub fn new(selector: GeneratorSelector) -> Self {
        Self {
            selector,
            state: NodeState::default(),
        }
    }

    fn apply_volume_and_pan(state: &NodeState, buffer: &mut Buffer, channel_index: usize) {
        let pan_mult = pan_multipliers(state.meta.pan)[channel_index];
        for x in buffer.iter_mut() {
            *x *= pan_mult * state.meta.volume;
        }
    }
}

impl Node<ProcessContext> for SimpleWaveGeneratorNode {
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
                        state.voice.eg.note_on();
                        state.voice.eg.set_envelope(state.config.envelope.clone());
                        // TODO: update config dynamically, not just when starting a new note.
                        state.voice.source = Some(SimpleWaveSource::new(
                            note_event.pitch_name.into(),
                            state.config.clone(),
                        ));
                    }
                    NoteEventType::Off => {
                        log::info!(
                            "Note off event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        // TODO: check against start/stop time too?
                        if let Some(source) = &state.voice.source {
                            if source.same_pitch(note_event.pitch_name) {
                                state.voice.eg.note_off();
                            }
                        }
                    }
                }
            }

            let amp = state.voice.eg.next().unwrap_or(0.0);
            let wave = state
                .voice
                .source
                .as_mut()
                .map(|it| it.next().unwrap_or(0.0))
                .unwrap_or(0.0);

            buffer[i] = amp * wave;
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume_and_pan(state, out_buf, channel_index);
        }
    }
}

pub struct SimpleWaveSource {
    // TODO: recycle the wave cache?
    // Currently it's re-created each time the note changes.
    cache: WaveCache,
    freq: Freq,
    config: SimpleWaveConfig,
    sample_index: usize,
}

impl SimpleWaveSource {
    pub fn new(freq: Freq, config: SimpleWaveConfig) -> Self {
        Self {
            cache: WaveCache::default(),
            freq,
            config,
            sample_index: 0,
        }
    }

    pub fn same_pitch(&self, pitch: PitchName) -> bool {
        self.freq == pitch.into()
    }
}

impl Iterator for SimpleWaveSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let detunes = linspace(
            -self.config.detune_cents,
            self.config.detune_cents,
            self.config.osc_count,
        );

        let mut output = 0.0;

        for detune in detunes {
            let freq = self.freq * detune_multiplier(detune);
            let step = freq / (SAMPLE_RATE as f32);
            let phase = ((self.sample_index as f32) * step) % 1.0;

            let key = WaveKey {
                kind: self.config.wave,
                aa: self.config.anti_aliasing_mode,
                freq: self.freq.into(),
            };
            if self.config.detune_cents != 0.0 {
                output += self.cache.get(&key, phase) / (self.config.osc_count as f32).sqrt();
            } else {
                output += self.cache.get(&key, phase) / self.config.osc_count as f32;
            }
        }

        self.sample_index += 1;
        Some(output)
    }
}
