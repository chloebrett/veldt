use super::{
    AudioBuffer, AudioProcessor, BUFFER_SIZE, EMPTY_BUFFER, PlaybackMessage, PlaybackPosition,
    PlaybackState, PlaybackUpdate,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use log::error;
use mesic::SAMPLE_RATE;
use mesic::graph::RenderGraph;
use shared::model::Project;
use std::sync::{Arc, Mutex};
use wasm_thread::JoinHandle;

pub struct AudioPlayer {
    // The render graph, if we haven't given it to the processing thread yet.
    // If this is 'None', this just means it lives in the processing thread now.
    // It can't be communicated with directly if that's the case, so pass PlaybackMessages to
    // indirectly manipulate it instead.
    // This field will only be Some while we're waiting to init the processing thread.
    // We need to pass this exact graph to the thread, instead of creating a new one,
    // because it has a reference to a StoreData channel receiver.
    graph: Option<RenderGraph>,

    // Messages sent from processor -> player.
    audio_tx: Sender<AudioBuffer>,
    audio_rx: Receiver<AudioBuffer>,

    // Messages sent from UI -> processor.
    playback_tx: Sender<PlaybackMessage>,
    playback_rx: Receiver<PlaybackMessage>,

    // Messages sent from processor -> UI.
    update_tx: Sender<PlaybackUpdate>,
    update_rx: Receiver<PlaybackUpdate>,

    stream: Option<Stream>,
    processor_thread: Option<JoinHandle<()>>,
    pub state: PlaybackState,
    is_looping: bool,
    pub position: PlaybackPosition,

    // Delay from the processing end.
    // Updated by listening for events from the processing thread.
    // No mutex needed, because we're already using a channel.
    buffer_delay: usize,

    // Delay from the audio playback end.
    // Needs to be a mutex because it's written from a static JS callback (the data callback
    // for the AudioContext).
    output_delay: Arc<Mutex<usize>>,
}

impl AudioPlayer {
    pub fn new(graph: RenderGraph) -> Self {
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(BUFFER_SIZE);
        let (playback_tx, playback_rx) = crossbeam_channel::unbounded();
        let (update_tx, update_rx) = crossbeam_channel::unbounded();

        Self {
            graph: Some(graph),
            audio_tx,
            audio_rx,
            playback_tx,
            playback_rx,
            update_tx,
            update_rx,
            stream: None,
            processor_thread: None,
            state: PlaybackState::Pause,
            is_looping: false,
            position: PlaybackPosition { samples: 0 },
            buffer_delay: 0,
            output_delay: Arc::new(Mutex::new(0)),
        }
    }

    /// Current position in playback, accounting for delay.
    pub fn effective_pos(&self) -> usize {
        // Output delay is irrelevant if we're not currently playing audio.
        let output_delay = if self.state == PlaybackState::Play {
            *self.output_delay.lock().unwrap()
        } else {
            0
        };

        // Don't overflow backwards.
        if self.buffer_delay + output_delay > self.position.samples {
            return 0;
        }

        // Buffer delay still matters though, but if we're paused the buffer will rapidly become empty.
        self.position.samples - self.buffer_delay - output_delay
    }

    fn send(&self, message: PlaybackMessage) {
        self.playback_tx.try_send(message).unwrap();
    }

    pub fn set_project(&mut self, project: Box<Project>) {
        self.maybe_init();
        self.send(PlaybackMessage::SetProject(project));
    }

    pub fn set_audio(&mut self, audio: Vec<Stereo<f32>>) {
        self.maybe_init();
        self.send(PlaybackMessage::SetAudio(audio));
    }

    fn is_ready(&mut self) -> bool {
        self.stream.is_some()
    }

    /// Browsers will only let us create an AudioContext after the user has interacted with the page.
    /// So do the initialization lazily.
    fn maybe_init(&mut self) {
        // We only need to initialize once.
        if !self.is_ready() {
            self.init_processor();
            self.init_stream();
        }
    }

    /// Checks for any pending updates from the processor thread and saves them locally.
    pub fn maybe_update(&mut self) {
        if !self.is_ready() {
            return;
        }

        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                PlaybackUpdate::Pos(pos) => {
                    self.position = pos;
                }
                PlaybackUpdate::Delay(delay) => {
                    self.buffer_delay = delay;
                }
                PlaybackUpdate::State(state) => {
                    self.state = state;
                }
            }
        }
    }

    pub fn init_processor(&mut self) {
        log::info!(
            "Available parallelism: {:?}",
            wasm_thread::available_parallelism()
        );

        let mut processor = AudioProcessor::new(
            self.audio_tx.clone(),
            self.playback_rx.clone(),
            self.update_tx.clone(),
            self.is_looping,
            self.graph.take().expect("Expected a render graph!"),
        );

        // Currently no way to stop a thread once it's started.
        // Could add a "Stop"/"Terminate" state.
        self.processor_thread = Some(wasm_thread::spawn(move || {
            processor.run();
        }));
    }

    pub fn init_stream(&mut self) {
        let audio_rx = self.audio_rx.clone();

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();
        let config: &cpal::StreamConfig = &config.into();

        let err_fn = |err| error!("an error occurred on stream: {}", err);
        if config.channels != 2 {
            panic!("Expected 2 output channels!");
        }

        let output_delay = self.output_delay.clone();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], info: &OutputCallbackInfo| {
                    // Pass info about the output latency back to the audio player.
                    // The latency info is used to accurately render the playback position.
                    let timestamp = info.timestamp();
                    if let Some(delay) = timestamp.playback.duration_since(&timestamp.callback) {
                        // It's fine to unlock the mutex every time here,
                        // because the data callback is only every N=1024 samples.
                        *output_delay.lock().unwrap() = to_samples(delay);
                    }

                    let received = audio_rx.try_recv().unwrap_or(EMPTY_BUFFER);
                    data.copy_from_slice(received.as_flattened());
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        self.stream = Some(stream);
    }

    pub fn is_looping(&self) -> bool {
        self.is_looping
    }

    pub fn set_looping(&mut self, value: bool) {
        self.is_looping = value;
        self.send(PlaybackMessage::Loop(value));
    }

    pub fn play(&mut self) {
        self.maybe_init();
        if self.state == PlaybackState::Play {
            return;
        }

        // If finished, restart.
        if self.state == PlaybackState::Finished {
            self.seek(0);
        }

        self.state = PlaybackState::Play;
        self.send(PlaybackMessage::State(self.state));

        // Note: we don't actually play/pause the stream in the AudioContext, other than the initial "play" call.
        // Calling play/pause multiple times has weird behaviour which might be a CPAL bug: subsequent pause/play
        // calls cause greater and greater delays between the audio callback being fired and actual playback.
        // This can be verified by logging
        // time_at_start_of_buffer - ctx_handle.current_time()
        // just before source.start_with_when is called in cpal::src::host::webaudio (mod.rs).
        // If you add this log then call play/pause on the audio stream several times,
        // each call causes a greater and greater delay (up to several seconds).
    }

    pub fn pause(&mut self) {
        self.maybe_init();
        if self.state == PlaybackState::Pause {
            return;
        }

        self.state = PlaybackState::Pause;
        self.send(PlaybackMessage::State(self.state));

        // Note: we don't actually play/pause the stream in the AudioContext, other than the initial "play" call.
        // See notes in the `play` method.
    }

    pub fn seek(&mut self, samples: usize) {
        self.position = PlaybackPosition { samples };
        self.send(PlaybackMessage::Seek(self.position));
    }
}

fn to_samples(duration: std::time::Duration) -> usize {
    duration.mul_f32(SAMPLE_RATE as f32).as_secs() as usize
}
