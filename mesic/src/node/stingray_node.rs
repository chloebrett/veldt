use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::consts::CHANNEL_COUNT;
use crate::envelope::EnvelopeGenerator;
use crate::eq::eq_filter;
use crate::eq::filter::Filter;
use crate::graph::{NoteEvent, NoteEventType, ProcessContext};
use crate::lfo::LfoGenerator;
use crate::maths::linspace;
use crate::wave::detune_multiplier;
use crate::wave_cache::{WaveCache, WaveKey};
use dasp_frame::Stereo;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{
    AntiAliasingMode, Generator, GeneratorInstance, GeneratorMeta, Oscillator, PitchName,
    StingrayConfig,
};
use shared::types::{Freq, KnobPosition, Volume};
use state::GeneratorSelector;
use std::collections::HashMap;

pub struct StingrayNode {
    selector: GeneratorSelector,
    state: NodeState,
    cache: WaveCache,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    config: StingrayConfig,
    meta: GeneratorMeta,
    voices: HashMap<String, Voice>,
    filter_left: Filter,
    filter_right: Filter,
}

struct Voice {
    egs: [EnvelopeGenerator; 3],
    sources: [StingrayWaveSource; 3],
    lfos: [LfoGenerator; 3],
}

impl Default for NodeState {
    fn default() -> Self {
        let config = StingrayConfig::default();
        Self {
            config: config.clone(),
            meta: GeneratorMeta::default(),
            voices: HashMap::new(),
            filter_left: eq_filter(&config.lpf),
            filter_right: eq_filter(&config.lpf),
        }
    }
}

impl NodeState {
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        if let GeneratorInstance {
            it: Generator::Stingray(config),
            meta,
            ..
        } = &payload.store.select(&selector)
        {
            if self.config != *config {
                if self.config.lpf != config.lpf {
                    // TODO: don't re-create the whole filter, just update
                    // the coefficients. Keep the ring buffer as is.
                    // Filter should have an update() method that takes some kind of config
                    // object.
                    self.filter_left = eq_filter(&config.lpf);
                    self.filter_right = eq_filter(&config.lpf);
                }
                if self.config.lfos != config.lfos {
                    for voice in self.voices.values_mut() {
                        for (i, lfo) in voice.lfos.iter_mut().enumerate() {
                            lfo.config = config.lfos[i].clone();
                        }
                    }
                }

                if self.config.envelopes != config.envelopes {
                    for voice in self.voices.values_mut() {
                        for (i, eg) in voice.egs.iter_mut().enumerate() {
                            eg.set_envelope(config.envelopes[i].clone());
                        }
                    }
                }

                self.config = config.clone();
            }
            if self.meta != *meta {
                self.meta = meta.clone();
            }
        }
    }
}

impl StingrayNode {
    pub fn new(selector: GeneratorSelector) -> Self {
        Self {
            selector,
            state: NodeState::default(),
            cache: WaveCache::default(),
        }
    }

    // TODO: this logic is similar and shared with stingray and simple wave, probably should move
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

impl Node<ProcessContext> for StingrayNode {
    // TODO: stingray will eventually have different polyphony/monophone modes and polyphony limit and the process method will need to respect it.
    // Currently the process method assumes polyphony and is not constrained by a polyphony limit
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer], payload: &ProcessContext) {
        // Locations of the LFOs and LPF in the matrix.
        const LFO_ROW_START: usize = 3;
        const LPF_COL_START: usize = 3;

        let state = &mut self.state;
        state.update(payload, self.selector);

        let mut buffers = [Buffer::SILENT; 2];
        let GeneratorSelector(generator_id) = self.selector;

        if payload.stop_generators.get(&generator_id) == Some(&true) {
            log::info!("Stopped stingray: {:?}", generator_id);
            for voice in state.voices.values_mut() {
                for eg in voice.egs.iter_mut() {
                    eg.note_off();
                }
            }
        }

        // TODO: fix this, it's n^2 right now. (well, n*64).
        for i in 0..Buffer::LEN {
            let events: Vec<NoteEvent> = payload
                .note_events
                .get(&generator_id)
                .cloned()
                .unwrap_or(vec![])
                .into_iter()
                .filter(|it| it.sample_index == i)
                .collect();

            // Update LFOs here so we can use their values else where
            for voice in state.voices.values_mut() {
                for lfo in voice.lfos.iter_mut() {
                    lfo.next();
                }
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
                            let osc = state.config.oscillators[i].clone();

                            // TODO: update config dynamically, not just when starting a new note.
                            sources.push(StingrayWaveSource::new(note_event.pitch_name, osc));
                        }

                        let mut new_voice = Voice {
                            egs: state.config.envelopes.clone().map(EnvelopeGenerator::new),
                            sources: sources.try_into().unwrap(),
                            lfos: state.config.lfos.clone().map(LfoGenerator::new),
                        };

                        for eg in new_voice.egs.iter_mut() {
                            eg.note_on();
                        }

                        let new_voice_key = note_event.pitch_name.to_string();
                        state.voices.insert(new_voice_key, new_voice);
                    }
                    NoteEventType::Off => {
                        log::info!(
                            "Note off event! {:?} {:?}",
                            note_event.pitch_name,
                            state.config
                        );
                        // TODO: check against start/stop time too?
                        let voice_key = note_event.pitch_name.to_string();
                        if let Some(voice_to_turn_off) = state.voices.get_mut(&voice_key) {
                            for eg in voice_to_turn_off.egs.iter_mut() {
                                eg.note_off();
                            }
                            // state.voices.remove(&voice_key);
                        }
                    }
                }
            }

            for voice in state.voices.values_mut() {
                for (j, source) in voice.sources.iter_mut().enumerate() {
                    let mut lfo_value = 0.0;
                    let mut lfo_active = false;
                    // Access the column for this oscillator in the matrix
                    for k in 0..state.config.lfos.len() {
                        // Get matrix value for this oscillator and LFO
                        let matrix_value = state
                            .config
                            .matrix
                            .get(k + LFO_ROW_START, j)
                            .map_or(0.0, |cell_ref| (*cell_ref).into());

                        if matrix_value != 0.0 {
                            lfo_active = true;
                        }

                        lfo_value += voice.lfos[k].current_value * matrix_value;
                    }

                    // Envelope-oscillator modulation
                    let wave = source.next(&mut self.cache, lfo_value, lfo_active);
                    let mut amp_mod = 0.0;

                    // Iterate through each envelope and accumulate modulation
                    for eg_idx in 0..voice.egs.len() {
                        let matrix_value = state
                            .config
                            .matrix
                            .get(eg_idx, j)
                            .map_or(0.0, |cell_ref| (*cell_ref).into());

                        if matrix_value != 0.0 {
                            let amp = voice.egs[eg_idx].next().unwrap_or(0.0) * matrix_value;
                            amp_mod += amp;
                        }
                    }

                    // If no modulation, set to 1.0 to play sample normally
                    let amp_mod = if amp_mod == 0.0 { 1.0 } else { amp_mod };

                    buffers[0][i] += amp_mod * wave[0];
                    buffers[1][i] += amp_mod * wave[1];
                }
            }

            // LPF modulation
            const LPF_MIN_FREQ: f32 = 20.0;
            const LPF_MAX_FREQ: f32 = 20000.0;

            let mut lfo_value = 0.0;

            // The modified LPF frequency should go to max freq at LFO value 1.0 and min freq at -1.0.
            let mut new_lpf_freq = state.config.lpf.fc + (LPF_MAX_FREQ - LPF_MIN_FREQ) * lfo_value;

            // Normalize cutoff frequency
            let mut env_mod = 0.0;

            for voice in state.voices.values_mut() {
                // Applying the LFO to the LPF
                // This implementation of the LFO LPF relation is based on the the ableton synth version
                // https://learningsynths.ableton.com/en/playground

                for k in 0..state.config.lfos.len() {
                    // Get matrix value for this LPF and LFO
                    let matrix_value = state
                        .config
                        .matrix
                        .get(k + LFO_ROW_START, LPF_COL_START)
                        .map_or(0.0, |cell_ref| (*cell_ref).into());

                    lfo_value += voice.lfos[k].current_value * matrix_value;
                }

                // Applying envelopes to LPF
                for eg_idx in 0..voice.egs.len() {
                    let matrix_value = state
                        .config
                        .matrix
                        .get(eg_idx, LPF_COL_START)
                        .map_or(0.0, |cell_ref| (*cell_ref).into());

                    let eg_val = voice.egs[eg_idx].peek();

                    // Accumulate modulation scaled by matrix value
                    env_mod += eg_val * matrix_value;
                }
            }
            // If no modulation, set to 1 so that sound is not cut off
            if env_mod == 0.0 {
                env_mod = 1.0;
            }

            new_lpf_freq = LPF_MIN_FREQ + env_mod * (new_lpf_freq - LPF_MIN_FREQ);

            new_lpf_freq = new_lpf_freq.clamp(LPF_MIN_FREQ, LPF_MAX_FREQ);

            let mut new_eq_config = state.config.lpf.clone();
            new_eq_config.fc = new_lpf_freq.into();

            state.filter_left.update_config(new_eq_config.clone());
            state.filter_right.update_config(new_eq_config.clone());
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffers[channel_index]);
            let meta = &state.meta;
            Self::apply_volume_and_pan(out_buf, channel_index, meta.volume, meta.pan);
        }

        if state.config.lpf_on {
            state.filter_left.apply(&mut output[0]);
            state.filter_right.apply(&mut output[1]);
        }

        // Removing any voices that have finished playing
        let finished_keys: Vec<String> = state.voices
            .iter()
            .filter(|(_key, voice)| voice.egs.iter().all(|eg| eg.is_shutdown()))
            .map(|(key, _voice)| key.clone())
            .collect();
        for key in finished_keys {
            state.voices.remove(&key);
        }
    }
}

#[derive(Debug)]
pub struct StingrayWaveSource {
    pitch: PitchName,
    oscillator: Oscillator,
    sample_index: usize,
}

impl StingrayWaveSource {
    pub fn new(pitch: PitchName, oscillator: Oscillator) -> Self {
        Self {
            pitch,
            oscillator,
            sample_index: 0,
        }
    }

    fn next(&mut self, cache: &mut WaveCache, lfo_value: f32, lfo_active: bool) -> Stereo<f32> {
        let osc = &self.oscillator;
        let freq: Freq = self.pitch.into();
        let freq = freq * detune_multiplier(osc.coarse_detune + osc.fine_detune);

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
        for (i, &detune) in detunes.iter().enumerate() {
            let freq = freq * detune_multiplier(detune);
            let step = freq / (SAMPLE_RATE as f32);

            // Lessen the initial 'pop' of the sound when playing with unison.
            let phase = (phases[i] + (self.sample_index as f32) * step) % 1.0;

            let key = WaveKey {
                kind: osc.wave,
                // Always use additive anti-aliasing, it sounds much better for square/saw waves.
                aa: AntiAliasingMode::Additive,
                freq: freq.into(),
            };

            output_mono += cache.get(&key, phase);
        }

        // Add soft clipping to lessen the peaks in volume.
        if unison > 1 {
            output_mono = (output_mono / 0.95).tanh() * 0.95;
        }

        // Apply the low frequency oscillator to the output.
        // Currently only used for volume modulation.
        if lfo_active {
            output_mono *= lfo_value;
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
