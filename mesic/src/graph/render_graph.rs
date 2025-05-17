use super::{
    NoteEvent, NoteEventType, NoteTracker, PlaybackMode, ProcessContext, Processor, make_processor,
};
use crate::convert::beats_to_samples;
use crate::mixer::Mixer;
use dasp_frame::Stereo;
use dasp_graph::Buffer;
use shared::model::{PitchName, Project};
use shared::types::Beats;
use state::{Action, GeneratorSelector, Selector, StoreData};
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
    pending_note_events: Vec<Vec<NoteEvent>>,

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
            mixer: Mixer::new(project),
            processor: make_processor(),
            process_context: ProcessContext::new(store.clone()),
            rx,
            pending_note_events: vec![],
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
        self.mixer = Mixer::new(project);
    }

    fn update_store(&mut self) {
        // Update the store if there are actions to process.
        // TODO: there is a bug where new effects won't pick up these changes immediately,
        // and have to have their values tweaked first.
        // Investigate.
        let store = &mut self.process_context.store;
        while let Ok((selector, action)) = self.rx.try_recv() {
            store.update(&selector, &action);

            // Also update the graph topology by listening for the appropriate actions.
            // E.g. add/remove effect or generator.
            self.mixer.update(&selector, &action, store);
        }

        self.update_duration();
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

    fn pos_mut(&mut self) -> &mut usize {
        match self.process_context.playback_mode {
            PlaybackMode::Main => &mut self.main_playback_index,
            PlaybackMode::Preview => &mut self.preview_playback_index,
        }
    }
}

fn duration_ceil(project: &Project) -> Beats {
    let beats = project.duration();
    const BEATS_PER_BAR: f32 = 4.0;
    (beats / BEATS_PER_BAR).ceil() * BEATS_PER_BAR
}

impl Iterator for RenderGraph {
    type Item = Stereo<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.pos();

        if index % Buffer::LEN == 0 {
            self.update_store();
            self.update_notes();
            self.mixer
                .process(&mut self.processor, &self.process_context);
            self.process_context.main_seek_pos = None;
            self.process_context.preview_seek_pos = None;
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

        let left = buffers[0][index % Buffer::LEN];
        let right = buffers[1][index % Buffer::LEN];
        let output = Some([left, right]);

        *self.pos_mut() += 1;

        output
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
        let mut graph = RenderGraph::without_rx(&empty_store_data());
        graph.set_audio(&input);
        let output: Vec<[f32; 2]> = graph.collect();

        // Assert
        assert_eq!(output, input)
    }
}
