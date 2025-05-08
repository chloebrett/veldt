use super::pan_multipliers;
use crate::SAMPLE_RATE;
use crate::consts::CHANNEL_COUNT;
use crate::graph::ProcessContext;
use crate::wave::{WaveCache, WaveKey, WaveSource, detune_multiplier, linspace, multi_sum};
use dasp_frame::Stereo;
use dasp_graph::{Buffer, Input, Node};
use shared::model::{Generator, Oscillator, GeneratorInstance, GeneratorMeta, PitchName, SubSynthConfig, AntiAliasingMode};
use shared::types::{Beats, Freq, KnobPosition, Volume};
use state::GeneratorSelector;

pub struct SubSynthNode {
    selector: GeneratorSelector,
    state: NodeState,
}

/// State persisted between buffers.
/// Specific to this node.
struct NodeState {
    bpm: Beats,
    wave_source: WaveSource,
    config: SubSynthConfig,
    meta: GeneratorMeta,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            bpm: 0.0,
            wave_source: WaveSource::new(0.0),
            config: SubSynthConfig::default(),
            meta: GeneratorMeta::default(),
        }
    }
}

impl NodeState {
    // TODO: update logic is almost the same as the simple wave generator.
    // Should it be de-duplicated?
    fn update(&mut self, payload: &ProcessContext, selector: GeneratorSelector) {
        let project = &payload.store.project;
        let bpm = project.bpm;

        if bpm != self.bpm {
            self.bpm = bpm;
            self.wave_source = WaveSource::new(bpm);
        }

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

        for note in &payload.notes[generator_index] {
            // TODO: add envelopes once mod matrix is working
            // NOTE: for now, osc 1 -> maps to env 1
            let wave_source = &mut self.state.wave_source;
            let oscillators = &self.state.config.oscillators;
            let envelopes = &self.state.config.envelopes;
            let osc_buffers: Vec<Buffer> = oscillators
                .iter()
                .zip(envelopes.iter())
                .map(|(osc, envelope)| {
                    let mut buf = wave_source.osc_wave(
                        note.pitch_name.into(),
                        note.duration,
                        osc,
                        envelope,
                        note.samples_since_started,
                    );

                    for channel_index in 0..CHANNEL_COUNT {
                        Self::apply_volume_and_pan(&mut buf, channel_index, osc.volume, osc.pan);
                    }

                    buf
                })
                .collect();

            dasp_slice::add_in_place(&mut buffer, &multi_sum(&osc_buffers));
        }

        for (channel_index, out_buf) in output.iter_mut().enumerate() {
            out_buf.copy_from_slice(&buffer);
            let meta = &self.state.meta;
            Self::apply_volume_and_pan(out_buf, channel_index, meta.volume, meta.pan);
        }
    }
}

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
            output_mono += self.cache.get(&key, phase);
        }

        let mut output_stereo = [output_mono, output_mono];
        for channel_index in 0..CHANNEL_COUNT {
            let pan_mult = pan_multipliers(osc.pan)[channel_index];
            output_stereo[channel_index] *= pan_mult * osc.volume;
        }

        self.sample_index += 1;
        Some(output_stereo)
    }
}
