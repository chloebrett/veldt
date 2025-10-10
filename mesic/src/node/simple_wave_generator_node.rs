use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::envelope::EnvelopeGenerator;
use crate::graph::{NoteEvent, NoteEventType, ProcessContext};
use crate::maths::linspace;
use crate::wave::detune_multiplier;
use crate::wave_cache::{WaveCache, WaveKey};
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Generator, GeneratorInstance, GeneratorMeta, SimpleWaveConfig};
use shared::types::Freq;
use state::GeneratorSelector;
use std::collections::HashMap;

pub struct SimpleWaveGeneratorNode {
    selector: GeneratorSelector,
    state: NodeState,
    cache: WaveCache,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    config: SimpleWaveConfig,
    meta: GeneratorMeta,
    voices: HashMap<(i32, usize), Voice>,
}

struct Voice {
    eg: EnvelopeGenerator,
    source: SimpleWaveSource,
}

impl Default for NodeState {
    fn default() -> Self {
        let config = SimpleWaveConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            voices: HashMap::new(),
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
            cache: WaveCache::default(),
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
    // TODO: process method does not currently respect the polyphony limit, need to constrain voices/implement voice stealing
    // TODO: process method also assumes generator is ALWAYS set to polyphony, needs to respect polyphony/monophone modes
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        let state = &mut self.state;
        state.update(payload, self.selector);

        let mut buffer = Buffer::SILENT;
        let GeneratorSelector(generator_id) = self.selector;

        if payload.stop_generators.get(&generator_id) == Some(&true) {
            log::info!("Stopped SWG: {:?}", generator_id);
            for voice in state.voices.values_mut() {
                voice.eg.note_off();
            }
        }

        // TODO: fix this, it's n^2 right now. (well, n*64).
        for i in 0..buffer.len() {
            let events: Vec<NoteEvent> = payload
                .note_events
                .get(&generator_id)
                .cloned()
                .unwrap_or(vec![])
                .into_iter()
                .filter(|it| it.sample_index == i)
                .collect();

            for note_event in events {
                match &note_event.kind {
                    NoteEventType::On => {
                        log::info!(
                            "Note on event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        // TODO: update config dynamically, not just when starting a new note.
                        let new_voice_key = (
                            note_event.pitch_name.into(),
                            note_event.global_start_sample_index,
                        );
                        let mut new_voice = Voice {
                            eg: EnvelopeGenerator::new(state.config.envelope.clone()),
                            source: SimpleWaveSource::new(
                                note_event.pitch_name.into(),
                                state.config.clone(),
                            ),
                        };
                        new_voice.eg.note_on();
                        state.voices.insert(new_voice_key, new_voice);
                    }
                    NoteEventType::Off => {
                        log::info!(
                            "Note off event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        // TODO: check against start/stop time too.
                        // TODO: Right now if you delete a note on the note roll before it finishes playing, the note off event won't be processed so the deleted note will play forever
                        let voice_key = (
                            note_event.pitch_name.into(),
                            note_event.global_start_sample_index,
                        );
                        if let Some(voice_to_turn_off) = state.voices.get_mut(&voice_key) {
                            voice_to_turn_off.eg.note_off();
                        }
                    }
                    NoteEventType::Shutdown => {
                        log::info!(
                            "Note shutdown event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        // Immediately stop the voice without going through the release phase of the envelope.
                        let voice_key = (
                            note_event.pitch_name.into(),
                            note_event.global_start_sample_index,
                        );
                        state.voices.remove(&voice_key);
                    }
                }
            }

            let mut cumulative_wave_amp_product = 0.0;
            for voice in state.voices.values_mut() {
                let amp = voice.eg.next().unwrap_or(0.0);
                let wave = voice.source.next(&mut self.cache);
                cumulative_wave_amp_product += amp * wave;
            }

            buffer[i] = cumulative_wave_amp_product;

            // Removing any voices that have finished playing
            let keys_to_remove: Vec<(i32, usize)> = state
                .voices
                .iter()
                .filter_map(|(voice_key, voice)| {
                    if voice.eg.is_off() {
                        Some(voice_key.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for voice_key in keys_to_remove {
                state.voices.remove(&voice_key);
            }
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            Self::apply_volume_and_pan(state, out_buf, channel_index);
        }
    }
}

pub struct SimpleWaveSource {
    freq: Freq,
    config: SimpleWaveConfig,
    sample_index: usize,
}

impl SimpleWaveSource {
    fn new(freq: Freq, config: SimpleWaveConfig) -> Self {
        Self {
            freq,
            config,
            sample_index: 0,
        }
    }

    fn next(&mut self, cache: &mut WaveCache) -> f32 {
        let detune_cents = self.config.detune_cents;
        let unison = if detune_cents == 0.0 {
            1
        } else {
            self.config.osc_count
        };

        let detunes = linspace(-detune_cents, detune_cents, unison);

        // Evenly spaced phases for each unison wave.
        // Use unison + 1 because phase=1 is the same as phase=0.
        let phases = linspace(0.0, 1.0, unison + 1);

        let mut output = 0.0;
        let unison_amp = (unison as f32).recip();
        for (i, &detune) in detunes.iter().enumerate() {
            let freq = self.freq * detune_multiplier(detune);
            let step = freq / (SAMPLE_RATE as f32);

            // Lessen the initial 'pop' of the sound when playing with unison.
            let phase = (phases[i] + (self.sample_index as f32) * step) % 1.0;

            let key = WaveKey {
                kind: self.config.wave,
                aa: self.config.anti_aliasing_mode,
                freq: self.freq.into(),
            };

            output += cache.get(&key, phase) * unison_amp;
        }

        // Add clipping to lessen the peaks in volume.
        if self.config.detune_cents > 0.0 && self.config.osc_count > 1 {
            output = (output / 0.95).tanh() * 0.95;
        }

        self.sample_index += 1;
        output
    }
}
