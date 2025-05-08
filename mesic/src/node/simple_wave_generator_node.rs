use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::envelope::EnvelopeGenerator;
use crate::graph::{ProcessContext, NoteEventType};
use crate::wave::{WaveCache, WaveKey, WaveSource};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Generator, GeneratorInstance, GeneratorMeta, PlacedNote, SimpleWaveConfig};
use shared::types::{Beats, Freq};
use state::GeneratorSelector;

struct Voice {
    eg: EnvelopeGenerator,
    note: Option<PlacedNote>,
}

pub struct SimpleWaveGeneratorNode {
    selector: GeneratorSelector,
    state: NodeState,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    bpm: Beats,
    wave_source: WaveSource,
    config: SimpleWaveConfig,
    meta: GeneratorMeta,
    voice: Voice,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = SimpleWaveConfig::default();
        Self {
            bpm: 0.0,
            wave_source: WaveSource::new(0.0),
            config: config.clone(),
            meta: GeneratorMeta::default(),
            voice: Voice {
                eg: EnvelopeGenerator::new(config.envelope.clone()),
                note: None,
            },
        }
    }
}

impl NodeState {
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        let project = &payload.store.project;
        let bpm = project.bpm;

        if bpm != self.bpm {
            self.bpm = bpm;
            self.wave_source = WaveSource::new(bpm);
        }

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
            for note_event in &payload.note_events[generator_index] {
                if note_event.sample_index == i {
                    log::info!("Sample index match! {i} {:?}", note_event);
                    match &note_event.kind {
                        NoteEventType::On { note } => {
                            state.voice.eg.note_on();
                        }
                        NoteEventType::Off => {
                            state.voice.eg.note_off();
                        }
                    }
                }
            }

            let amp = state.voice.eg.next();
        }
        for note_event in &payload.note_events[generator_index] {
            log::info!("{:?}", note_event);
        }

        for note in &payload.notes[generator_index] {
            dasp_slice::add_in_place(
                &mut buffer,
                &state.wave_source.unison_wave(
                    note.pitch_name.into(),
                    note.duration,
                    &state.config,
                    note.samples_since_started,
                ),
            );
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume_and_pan(state, out_buf, channel_index);
        }
    }
}

pub struct SimpleWaveSource {
    // TODO: recycle the wave cache?
    cache: WaveCache,
    freq: Freq,
    config: SimpleWaveConfig,
    sample_index: usize,
}

pub struct Unison {
    pub detune_cents: f32,
    pub osc_count: usize,
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
}

impl Iterator for SimpleWaveSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: support unison again.

        let step = self.freq / (SAMPLE_RATE as f32);
        let phase = ((self.sample_index as f32) * step) % 1.0;

        self.sample_index += 1;

        let key = WaveKey {
            kind: self.config.wave,
            aa: self.config.anti_aliasing_mode,
            freq: self.freq.into(),
        };
        Some(self.cache.get(&key, phase))
    }
}
