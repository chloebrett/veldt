use super::{Mixer, ProcessContext, Processor, make_processor};
use crate::wave::beats_to_samples;
use dasp_frame::Stereo;
use dasp_graph::Buffer;
use shared::model::Project;
use state::{Action, Selector};
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

    pub fn set_from_audio(&mut self, audio: Vec<Stereo<f32>>) {
        self.mixer = Mixer::from_audio(&audio);
        self.sample_count = audio.len();
    }

    /// Initializes the graph from a project instance.
    /// Not idempotent! Only call this on a fresh RenderGraph. (either new or call clear_nodes).
    /// This is mostly an interim method until we get action receiving working properly.
    pub fn set_from_project(&mut self, project: &Project) {
        self.mixer = Mixer::from_project(project);
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

    /// Creates a graph that plays the buffer contained in a Vec.
    pub fn from_vec(vec: Vec<Stereo<f32>>) -> Self {
        let mut graph = Self::default();
        graph.set_from_audio(vec);
        graph
    }

    fn update_store(&mut self) {
        // Update the store if there are actions to process.
        let store = &mut self.process_context.store;
        if let Some(rx) = &self.rx {
            while let Ok((selector, action)) = rx.try_recv() {
                // TODO: also update graph topology by listening for the appropriate actions.
                // E.g. add/remove effect or generator.
                store.update(&selector, &action);
            }
        }
    }
}

impl Iterator for RenderGraph {
    type Item = Stereo<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_store();

        if self.processed_samples_count % Buffer::LEN == 0 {
            self.mixer
                .process(&mut self.processor, &self.process_context);
            self.process_context.seek_pos = None;
        }

        if self.processed_samples_count >= self.sample_count {
            return None;
        }

        let buffers = &self.mixer.output_buffers();

        let left = buffers[0][self.processed_samples_count % Buffer::LEN];
        let right = buffers[1][self.processed_samples_count % Buffer::LEN];
        let output = Some([left, right]);

        self.processed_samples_count += 1;
        output
    }

    // TODO: implement size_hint or SizedIterator to make collection more efficient.
}

#[cfg(test)]
mod tests {
    use crate::SAMPLE_RATE;
    use shared::model::{
        AdsrEnvelope, AntiAliasingMode, DelayConfig, Effect, EffectInstance, EffectMeta, EqConfig,
        EqType, Generator, GeneratorInstance, GeneratorMeta, MixerChannel, ModDelayConfig,
        ModMatrix, Note, PitchName, PlacedNote, Placement, PlacementType, ScaleValue,
        SimpleWaveConfig, Track, TrackPlacement, WaveType,
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
            mixer: vec![make_mixer_channel()],
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
    fn basic_render_graph_renders_something() {
        // Arrange
        let mut graph = RenderGraph::default();
        graph.set_from_project(&make_project());
        // Assert
        assert!(rms(graph) > 0.0)
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
