use super::{
    DrumNoteTracker, NoteEvent, NoteEventType, NoteTracker, PlaybackMode, ProcessContext,
    Processor, make_processor,
};
use crate::convert::beats_to_samples;
use crate::mixer::{GraphDebugInfo, Mixer};
use crossbeam_channel::Sender;
use dasp_frame::Stereo;
use dasp_graph::Buffer;
use shared::model::{GeneratorId, PitchName, PlacementType, Project};
use shared::types::Beats;
use state::{Action, GeneratorSelector, Selector, StoreData, TypeField};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

/// Wraps a Mixer (which in turn wraps a Graph) to add processing/iteration, seeking, and listening
/// for updates to the Store.
pub struct RenderGraph {
    mixer: Mixer,
    processor: Processor,

    // Contains a copy of the project.
    // Updated based on actions from the main store at each buffer cycle.
    process_context: ProcessContext,
    // Receives actions from the main store and applies to mesic store.
    rx: Receiver<(Selector, Action)>,

    // Pending note on/off events sent from UI (e.g. from interacting with piano).
    pending_note_events: HashMap<GeneratorId, Vec<NoteEvent>>,

    // Playback state.
    main_playback_len: usize,
    main_playback_index: usize,
    preview_playback_len: usize,
    preview_playback_index: usize,
}

impl RenderGraph {
    pub fn new(store: &StoreData, rx: Receiver<(Selector, Action)>) -> Self {
        let project = &store.project;
        let main_playback_len = beats_to_samples(duration_ceil(project), project.bpm) as usize;

        Self {
            mixer: Mixer::new(project, None),
            processor: make_processor(),
            process_context: ProcessContext::new(store.clone()),
            rx,
            pending_note_events: HashMap::new(),
            main_playback_len,
            main_playback_index: 0,
            preview_playback_len: 0,
            preview_playback_index: 0,
        }
    }

    pub fn without_rx(store: &StoreData) -> Self {
        let (_tx, rx) = std::sync::mpsc::channel();
        Self::new(store, rx)
    }

    pub fn set_audio(&mut self, audio: &[Stereo<f32>]) {
        self.process_context.preview_buffer = audio.to_owned();
        self.preview_playback_len = audio.len();
        self.preview_playback_index = 0;
        self.process_context.playback_mode = PlaybackMode::Preview;
    }

    pub fn pos(&self) -> usize {
        match self.process_context.playback_mode {
            PlaybackMode::Main => self.main_playback_index,
            PlaybackMode::Preview => self.preview_playback_index,
        }
    }

    pub fn seek(&mut self, samples: usize) {
        match self.process_context.playback_mode {
            PlaybackMode::Main => {
                self.main_playback_index = samples;
                self.process_context.main_seek_pos = Some(samples);
            }
            PlaybackMode::Preview => {
                self.preview_playback_index = samples;
                self.process_context.preview_seek_pos = Some(samples);
            }
        }
    }

    pub fn recreate_mixer(&mut self) {
        self.update_store();
        let project = &self.process_context.store.project;
        self.main_playback_index = 0;
        self.preview_playback_index = 0;
        self.process_context.playback_mode = PlaybackMode::Main;

        let debug_tx = self.mixer.get_debug_tx();
        self.mixer = Mixer::new(project, debug_tx);
    }

    pub fn set_debug_tx(&mut self, debug_tx: Sender<GraphDebugInfo>) {
        log::info!("Set debug tx");
        self.mixer.set_debug_tx(debug_tx);
    }

    fn update_store(&mut self) {
        // Update the store if there are actions to process.
        // TODO: there is a bug where new effects won't pick up these changes immediately,
        // and have to have their values tweaked first.
        // Investigate.
        while let Ok((selector, action)) = self.rx.try_recv() {
            // Stop generators when a track changes its generator index.
            // This needs to run before the store update, as we reference the previous state of the
            // store.
            self.maybe_stop_generator(&selector, &action);

            let store = &mut self.process_context.store;
            store.update(&selector, &action);

            // Also update the graph topology by listening for the appropriate actions.
            // E.g. add/remove effect or generator.
            self.mixer.update(&selector, &action, store);
        }

        self.update_duration();
    }

    // If a track placement changes its generator index, stop the old generator from playing.
    fn maybe_stop_generator(&mut self, selector: &Selector, action: &Action) {
        let store = &self.process_context.store;
        let Selector::Placement(placement_id) = selector else {
            return;
        };

        let Action::SetChild(TypeField::GeneratorId(_)) = action else {
            return;
        };

        let PlacementType::Track(track_placement) = &store.project.placements[placement_id].kind
        else {
            return;
        };

        let prev_generator_id = track_placement.generator_id;
        let stop_generators = &mut self.process_context.stop_generators;
        stop_generators.insert(prev_generator_id, true);
        log::info!("Stopped generator: {:?}", stop_generators);
    }

    fn update_duration(&mut self) {
        let project = &self.process_context.store.project;
        self.main_playback_len = beats_to_samples(duration_ceil(project), project.bpm) as usize;
    }

    fn update_notes(&mut self) {
        self.process_context.note_events = NoteTracker::track(
            &self.process_context.store.project,
            self.main_playback_index,
        );

        self.process_context.drum_note_events = DrumNoteTracker::track(
            &self.process_context.store.project,
            self.main_playback_index,
        );

        // Load any events sent from the UI by the user.
        for (generator_id, event) in self.pending_note_events.clone().into_iter() {
            self.process_context
                .note_events
                .entry(generator_id)
                .or_default()
                .extend(event);
        }
        self.pending_note_events.clear();
    }

    /// Processes a note event sent by the user.
    /// Non-public to avoid exposing NoteEventType enum to hydric.
    fn note_event(
        &mut self,
        generator: GeneratorSelector,
        pitch_name: PitchName,
        kind: NoteEventType,
    ) {
        let GeneratorSelector(generator_id) = generator;
        self.pending_note_events
            .entry(generator_id)
            .or_default()
            .push(NoteEvent {
                kind,
                sample_index: 0,
                pitch_name,
                pitch_offset: 0.0,
                global_start_sample_index: 0,
            });
        log::info!("Pending: {:?}", self.pending_note_events);
    }

    pub fn note_on(&mut self, generator: GeneratorSelector, pitch_name: PitchName) {
        self.note_event(generator, pitch_name, NoteEventType::On);
    }

    pub fn note_off(&mut self, generator: GeneratorSelector, pitch_name: PitchName) {
        self.note_event(generator, pitch_name, NoteEventType::Off);
    }

    fn pos_mut(&mut self) -> &mut usize {
        match self.process_context.playback_mode {
            PlaybackMode::Main => &mut self.main_playback_index,
            PlaybackMode::Preview => &mut self.preview_playback_index,
        }
    }

    // Return only the main channel audio from the graph.
    pub fn collect_main(self) -> Vec<[f32; 2]> {
        self.map(|it| it[0]).collect()
    }
}

// TODO: account for sample placements.
fn duration_ceil(project: &Project) -> Beats {
    let beats = project.duration();
    const BEATS_PER_BAR: f32 = 4.0;
    (beats / BEATS_PER_BAR).ceil() * BEATS_PER_BAR
}

impl Iterator for RenderGraph {
    type Item = Vec<Stereo<f32>>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.pos();
        self.process_context.playback_pos = index;

        if index % Buffer::LEN == 0 {
            self.update_store();
            self.update_notes();
            self.mixer
                .process(&mut self.processor, &self.process_context);
            self.process_context.main_seek_pos = None;
            self.process_context.preview_seek_pos = None;
            self.process_context.stop_generators.clear();
        }

        match self.process_context.playback_mode {
            PlaybackMode::Main => {
                if index >= self.main_playback_len {
                    return None;
                }
            }
            PlaybackMode::Preview => {
                if index >= self.preview_playback_len {
                    self.process_context.playback_mode = PlaybackMode::Main;
                    return None;
                }
            }
        }

        let buffers = &self.mixer.output_buffers();
        let output = buffers
            .iter()
            .map(|&buffer| {
                let left = buffer[0][index % Buffer::LEN];
                let right = buffer[1][index % Buffer::LEN];
                [left, right]
            })
            .collect();

        *self.pos_mut() += 1;

        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::SAMPLE_RATE;
    use shared::model::{PitchName, ScaleValue};

    use crate::testing::store::empty_store_data;
    use shared::types::Freq;

    use super::*;

    #[test]
    fn empty_render_graph_renders_nothing() {
        // TODO Fix. This test does not terminate.
        let graph = RenderGraph::without_rx(&empty_store_data());
        // Iterator should be empty.
        let output: Vec<Stereo<f32>> = graph.collect_main();
        assert!(output.is_empty())
    }

    #[test]
    #[ignore]
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
        let mut graph = RenderGraph::without_rx(&empty_store_data());
        graph.set_audio(&input);
        let output: Vec<Stereo<f32>> = graph.collect_main();

        // Assert
        assert_eq!(output, input)
    }
}
