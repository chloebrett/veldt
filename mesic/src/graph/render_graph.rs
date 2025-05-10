use super::{NoteEvent, NoteEventType, NoteTracker, ProcessContext, Processor, make_processor};
use crate::mixer::Mixer;
use crate::wave::beats_to_samples;
use dasp_frame::Stereo;
use dasp_graph::Buffer;
use shared::model::{PitchName, Project};
use state::{Action, GeneratorSelector, Selector};
use std::sync::mpsc::Receiver;

/// Wraps a Mixer (which in turn wraps a Graph) to add processing/iteration, seeking, and listening
/// for updates to the Store.
pub struct RenderGraph {
    mixer: Mixer,
    sample_count: usize,
    processor: Processor,

    // Contains a copy of the project.
    // Updated based on actions from the main store at each buffer cycle.
    process_context: ProcessContext,
    // Receives actions from the main store and applies to mesic store.
    rx: Option<Receiver<(Selector, Action)>>,

    // For iteration.
    processed_samples_count: usize,

    // Pending note on/off events sent from UI (e.g. from interacting with piano).
    pending_note_events: Vec<Vec<NoteEvent>>,

    pub is_playing: bool,
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self {
            mixer: Mixer::empty(),
            sample_count: 0,
            processor: make_processor(),
            process_context: ProcessContext::default(),
            rx: None,
            processed_samples_count: 0,
            pending_note_events: vec![],
            is_playing: false,
        }
    }
}

impl RenderGraph {
    /// Deletes all nodes from the graph.
    pub fn clear_nodes(&mut self) {
        self.mixer = Mixer::empty();

        // Reset counters.
        self.sample_count = 0;
        self.processor = make_processor();
        self.processed_samples_count = 0;

        // Keep the process context and rx because they contain the store.
    }

    pub fn set_audio(&mut self, audio: &Vec<Stereo<f32>>) {
        self.process_context.preview_buffer = audio.clone();
        self.sample_count = audio.len();
    }

    pub fn set_from_store(&mut self) {
        self.update_store();
        let project = self.process_context.store.project.clone();
        self.mixer = Mixer::from_project(&project);
        self.sample_count = beats_to_samples(*project.duration(), project.bpm) as usize;
    }

    pub fn pos(&self) -> usize {
        self.processed_samples_count
    }

    pub fn seek(&mut self, samples: usize) {
        self.processed_samples_count = samples;
        self.process_context.seek_pos = Some(samples);
    }

    pub fn set_receiver(&mut self, receiver: Receiver<(Selector, Action)>) {
        self.rx = Some(receiver);
    }

    fn update_store(&mut self) {
        // Update the store if there are actions to process.
        // TODO: there is a bug where new effects won't pick up these changes immediately,
        // and have to have their values tweaked first.
        // Investigate.
        let store = &mut self.process_context.store;
        if let Some(rx) = &self.rx {
            while let Ok((selector, action)) = rx.try_recv() {
                store.update(&selector, &action);

                // Also update the graph topology by listening for the appropriate actions.
                // E.g. add/remove effect or generator.
                self.mixer.update(&selector, &action, store);
            }
        }
    }

    fn update_notes(&mut self) {
        self.process_context.note_events = NoteTracker::track(
            &self.process_context.store.project,
            self.processed_samples_count,
            /* include_on_events= */ self.is_playing,
        );

        // Load any events sent from the UI by the user.
        for (i, event) in self.pending_note_events.clone().into_iter().enumerate() {
            self.process_context.note_events[i].extend(event);
        }
        self.pending_note_events = vec![];
    }

    /// Processes a note event sent by the user.
    /// Non-public to avoid exposing NoteEventType enum to hydric.
    fn note_event(
        &mut self,
        generator: GeneratorSelector,
        pitch_name: PitchName,
        kind: NoteEventType,
    ) {
        let GeneratorSelector(generator_index) = generator;
        // Note: this pattern will become a bit inefficient if there are a lot of generators.
        while self.pending_note_events.len() <= generator_index {
            self.pending_note_events.push(vec![]);
        }
        self.pending_note_events[generator_index].push(NoteEvent {
            kind,
            sample_index: 0,
            pitch_name,
        });
        log::info!("Pending: {:?}", self.pending_note_events);
    }

    pub fn note_on(&mut self, generator: GeneratorSelector, pitch_name: PitchName) {
        self.note_event(generator, pitch_name, NoteEventType::On);
    }

    pub fn note_off(&mut self, generator: GeneratorSelector, pitch_name: PitchName) {
        self.note_event(generator, pitch_name, NoteEventType::Off);
    }
}

impl Iterator for RenderGraph {
    type Item = Stereo<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.processed_samples_count % Buffer::LEN == 0 {
            self.update_store();
            self.update_notes();
            self.mixer
                .process(&mut self.processor, &self.process_context);
            self.process_context.seek_pos = None;
        }

        if self.is_playing && (self.processed_samples_count >= self.sample_count) {
            return None;
        }

        let buffers = &self.mixer.output_buffers();

        let left = buffers[0][self.processed_samples_count % Buffer::LEN];
        let right = buffers[1][self.processed_samples_count % Buffer::LEN];
        let output = Some([left, right]);

        self.processed_samples_count += 1;

        output
    }
}

#[cfg(test)]
mod tests {
    use crate::SAMPLE_RATE;
    use shared::model::{
        self, AdsrEnvelope, AntiAliasingMode, DelayConfig, Effect, EffectInstance, EffectMeta,
        EqConfig, EqType, Generator, GeneratorInstance, GeneratorMeta, MixerChannel, MixerMatrix,
        ModDelayConfig, ModMatrix, Note, PitchName, PlacedNote, Placement, PlacementType,
        ScaleValue, SimpleWaveConfig, Track, TrackPlacement, WaveType,
    };

    use shared::types::Freq;

    use super::*;

    // Root Mean Squared to calculate if there is signal in output.
    fn rms(graph: RenderGraph) -> f32 {
        let mut left_sum = 0.0;
        let mut right_sum = 0.0;
        let mut count = 0;
        for [left, right] in graph {
            count += 1;
            left_sum += left.powi(2);
            right_sum += right.powi(2);
        }
        ((left_sum / count as f32 + right_sum / count as f32) / 2.0).sqrt()
    }

    fn make_project() -> Project {
        Project {
            name: "test".into(),
            tracks: vec![make_track()],
            placements: vec![make_placement()],
            samples: vec![],
            generators: vec![GeneratorInstance {
                it: Generator::SimpleWave(make_simple_wave_config()),
                meta: make_generator_meta(),
            }],
            mixer: model::Mixer {
                matrix: MixerMatrix::with_channels(1),
                channels: vec![make_mixer_channel()],
            },
            bpm: 120.0,
            mod_matrix: ModMatrix::default(),
        }
    }

    fn make_track() -> Track {
        Track {
            notes: vec![PlacedNote {
                note: Note {
                    pitch_name: PitchName {
                        scale_value: ScaleValue::C,
                        octave: 4,
                    },
                    beats: 1.0,
                },
                offset: 0.0.into(),
            }],
            offset: 0.0.into(),
        }
    }

    fn make_placement() -> Placement {
        Placement {
            kind: PlacementType::Track(TrackPlacement {
                track_index: 0,
                generator_index: 0,
            }),
            offset: 0.0.into(),
            clipped_duration: None,
            visual_placement: 0,
        }
    }

    fn make_simple_wave_config() -> SimpleWaveConfig {
        SimpleWaveConfig {
            wave: WaveType::Sine,
            envelope: AdsrEnvelope {
                attack: 0.1,
                decay: 0.1,
                sustain: 0.8,
                release: 0.1,
            },
            osc_count: 4,
            detune_cents: 5.0,
            anti_aliasing_mode: AntiAliasingMode::Off,
            oversample_factor: 2,
        }
    }

    fn make_generator_meta() -> GeneratorMeta {
        GeneratorMeta {
            volume: 1.0,
            mute: false,
            pan: 0.0,
            mixer_channel: 0,
        }
    }

    fn make_mixer_channel() -> MixerChannel {
        MixerChannel {
            volume: 1.0,
            effects: vec![
                EffectInstance {
                    it: Effect::SimpleEq(EqConfig {
                        kind: EqType::SimpleResonator,
                        fc: 1000.0,
                        q: 1.0,
                        gain: 0.0,
                    }),
                    meta: EffectMeta {
                        wet: 1.0,
                        mute: false,
                    },
                },
                EffectInstance {
                    it: Effect::Delay(DelayConfig {
                        delay_ms: 250.0,
                        feedback: 0.5,
                    }),
                    meta: EffectMeta {
                        wet: 0.5,
                        mute: false,
                    },
                },
                EffectInstance {
                    it: Effect::ModDelay(ModDelayConfig {
                        min_depth: 100,
                        max_depth: 200,
                        freq: 10.0,
                        lfo_type: WaveType::Triangle,
                    }),
                    meta: EffectMeta {
                        wet: 0.5,
                        mute: false,
                    },
                },
            ],
        }
    }

    #[test]
    fn empty_render_graph_renders_nothing() {
        let graph = RenderGraph::default();
        // Iterator should be empty.
        let output: Vec<[f32; 2]> = graph.collect();
        assert!(output.is_empty())
    }

    #[test]
    fn graph_built_from_sample_renders_correctly() {
        // Arrange
        let pitch = PitchName {
            scale_value: ScaleValue::A,
            octave: 4,
        };
        let samples = 120;
        let input: Vec<Stereo<f32>> = (0..samples as usize)
            .map(|it| {
                let freq: Freq = pitch.into();
                let value = (it as f32 / SAMPLE_RATE as f32 * freq).sin();
                [value, value]
            })
            .collect();

        // Act
        let graph = RenderGraph::from_vec(input.clone());
        let output: Vec<[f32; 2]> = graph.collect();

        // Assert
        assert_eq!(output, input)
    }
}
