use super::{
    AudioProcessor, BUFFER_SIZE, PlaybackMessage, PlaybackPosition, PlaybackState, PlaybackUpdate,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use log::error;
use mesic::SAMPLE_RATE;
use shared::model::Project;
use shared::types::Volume;
use std::sync::{Arc, Mutex};
use wasm_thread::JoinHandle;

pub struct AudioPlayer {
    // Messages sent from processor -> player.
    audio_tx: Sender<Stereo<f32>>,
    audio_rx: Receiver<Stereo<f32>>,

    // Messages sent from UI -> processor.
    playback_tx: Sender<PlaybackMessage>,
    playback_rx: Receiver<PlaybackMessage>,

    // Messages sent from processor -> UI.
    update_tx: Sender<PlaybackUpdate>,
    update_rx: Receiver<PlaybackUpdate>,

    stream: Option<Stream>,
    processor_thread: Option<JoinHandle<()>>,
    pub state: PlaybackState,
    pub position: PlaybackPosition,

    // Delay from the processing end.
    buffer_delay: usize,

    // Delay from the audio playback end.
    output_delay: Arc<Mutex<usize>>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(BUFFER_SIZE);
        let (playback_tx, playback_rx) = crossbeam_channel::unbounded();
        let (update_tx, update_rx) = crossbeam_channel::unbounded();

        AudioPlayer {
            audio_tx,
            audio_rx,
            playback_tx,
            playback_rx,
            update_tx,
            update_rx,
            stream: None,
            processor_thread: None,
            state: PlaybackState::Pause,
            position: PlaybackPosition { samples: 0 },
            buffer_delay: 0,
            output_delay: Arc::new(Mutex::new(0)),
        }
    }
}

impl AudioPlayer {
    /// Current position in playback, accounting for delay.
    pub fn effective_pos(&self) -> usize {
        // Output delay is irrelevant if we're not currently playing audio.
        let output_delay = if self.state == PlaybackState::Play {
            *self.output_delay.lock().unwrap()
        } else {
            0
        };

        // Buffer delay still matters though, but if we're paused the buffer will rapidly become empty.
        self.position.samples - self.buffer_delay - output_delay
    }

    fn send(&self, message: PlaybackMessage) {
        self.playback_tx.try_send(message).unwrap();
    }

    pub fn set_project(&mut self, project: Box<Project>, volume: Volume) {
        self.maybe_init();
        self.send(PlaybackMessage::SetProject(project, volume));
    }

    pub fn set_audio(&mut self, audio: Vec<Stereo<f32>>, volume: Volume) {
        self.maybe_init();
        self.send(PlaybackMessage::SetAudio(audio, volume));
    }

    fn is_ready(&mut self) -> bool {
        self.stream.is_some()
    }

    /// Browsers will only let us create an AudioContext after the user has interacted with the
    /// page.
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
        let channels = config.channels as usize;

        let output_delay = self.output_delay.clone();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], info: &OutputCallbackInfo| {
                    let timestamp = info.timestamp();
                    if let Some(delay) = timestamp.playback.duration_since(&timestamp.callback) {
                        *output_delay.lock().unwrap() = to_samples(delay);
                    }

                    for frame in data.chunks_mut(channels) {
                        let value = audio_rx.try_recv().unwrap_or([0.0; 2]);

                        frame[0] = value[0]; // left
                        frame[1] = value[1]; // right
                    }
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        self.stream = Some(stream);
    }

    pub fn play(&mut self) {
        self.maybe_init();
        if self.state == PlaybackState::Play {
            return;
        }

        self.state = PlaybackState::Play;
        self.send(PlaybackMessage::State(self.state));

        // Note: we don't actually play/pause the stream in the AudioContext, other than the initial "play" call.
        // Calling play/pause multiple times has weird behaviour which might be a CPAL bug: subsequent pause/play calls cause greater and greater delays
        // between the audio callback being fired and actual playback.
        // This can be verified by logging
        // time_at_start_of_buffer - ctx_handle.current_time()
        // just before source.start_with_when is called in cpal::src::host::webaudio (mod.rs).
        // Subsequent play/pause runs cause a greater and greater delay (up to several seconds).
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
