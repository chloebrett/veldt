use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::consts::CHANNEL_COUNT;
use crate::envelope::EnvelopeGenerator;
use crate::graph::{NoteEventType, ProcessContext};
use crate::maths::linspace;
use crate::wave::detune_multiplier;
use crate::wave_cache::{WaveCache, WaveKey};
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
    cache: WaveCache,
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
            cache: WaveCache::default(),
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

        let mut buffers = [Buffer::SILENT; 2];
        let GeneratorSelector(generator_index) = self.selector;

        // TODO: fix this, it's n^2 right now. (well, n*64).
        for i in 0..Buffer::LEN {
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
                    let wave = source.next(&mut self.cache);

                    buffers[0][i] += amp * wave[0];
                    buffers[1][i] += amp * wave[1];
                }
            }
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffers[channel_index]);
            let meta = &self.state.meta;
            Self::apply_volume_and_pan(out_buf, channel_index, meta.volume, meta.pan);
        }
    }
}

#[derive(Debug)]
pub struct SubSynthWaveSource {
    pitch: PitchName,
    oscillator: Oscillator,
    sample_index: usize,
}

impl SubSynthWaveSource {
    pub fn new(pitch: PitchName, oscillator: Oscillator) -> Self {
        Self {
            pitch,
            oscillator,
            sample_index: 0,
        }
    }

    pub fn same_pitch(&self, pitch: PitchName) -> bool {
        self.pitch == pitch
    }

    fn next(&mut self, cache: &mut WaveCache) -> Stereo<f32> {
        let osc = &self.oscillator;
        let freq: Freq = self.pitch.into();
        let freq = freq * detune_multiplier(osc.osc_detune);

        let unison = if osc.unison_detune == 0.0 {
            1
        } else {
            osc.osc_count
        };
        let detunes = linspace(-osc.unison_detune, osc.unison_detune, unison);

        // Evenly spaced phases for each unison wave.
        // Use unison + 1 because phase=1 is the same as phase=0.
        let phases = linspace(0.0, 1.0, unison + 1);

        let mut output_mono = 0.0;
        let unison_amp = (unison as f32).recip();
        for (i, &detune) in detunes.iter().enumerate() {
            let freq = freq * detune_multiplier(detune);
            let step = freq / (SAMPLE_RATE as f32);

            // Lessen the initial 'pop' of the sound when playing with unison.
            let phase = (phases[i] + (self.sample_index as f32) * step) % 1.0;

            let key = WaveKey {
                kind: osc.wave,
                aa: AntiAliasingMode::Off,
                freq: freq.into(),
            };

            output_mono += cache.get(&key, phase) * unison_amp;
        }

        // Add soft clipping to lessen the peaks in volume.
        if unison > 1 {
            output_mono = (output_mono / 0.95).tanh() * 0.95;
        }

        let mut output_stereo = [output_mono; CHANNEL_COUNT];
        for (channel_index, out) in output_stereo.iter_mut().enumerate() {
            let pan_mult = pan_multipliers(osc.pan)[channel_index];
            *out *= pan_mult * osc.volume;
        }

        self.sample_index += 1;
        output_stereo
    }
}
