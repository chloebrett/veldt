use super::{
    AudioProcessor, BUFFER_SIZE, PlaybackMessage, PlaybackPosition, PlaybackState, PlaybackUpdate,
};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use log::error;
use shared::model::Project;
use shared::types::Volume;
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

    // Delay caused by buffering.
    delay: usize,
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
            state: PlaybackState::Stop,
            position: PlaybackPosition { samples: 0 },
            delay: 0,
        }
    }
}

impl AudioPlayer {
    /// Current position in playback, accounting for delay.
    pub fn effective_pos(&self) -> usize {
        self.position.samples - self.delay
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
                    self.delay = delay;
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

        self.stream = Some(
            device
                .build_output_stream(
                    config,
                    move |data: &mut [f32], _| {
                        for frame in data.chunks_mut(channels) {
                            let value = audio_rx.try_recv().unwrap_or([0.0; 2]);

                            frame[0] = value[0]; // left
                            frame[1] = value[1]; // right
                        }
                    },
                    err_fn,
                    None,
                )
                .unwrap(),
        );
    }

    pub fn play(&mut self) {
        self.maybe_init();
        if self.state == PlaybackState::Play {
            return;
        }

        self.state = PlaybackState::Play;
        self.send(PlaybackMessage::State(self.state));
        self.stream.as_mut().unwrap().play().unwrap();
    }

    pub fn pause(&mut self) {
        self.maybe_init();
        if self.state == PlaybackState::Pause {
            return;
        }

        self.state = PlaybackState::Pause;
        self.send(PlaybackMessage::State(self.state));
        self.stream.as_mut().unwrap().pause().unwrap();
    }

    pub fn seek(&mut self, samples: usize) {
        self.position = PlaybackPosition { samples };
        self.send(PlaybackMessage::Seek(self.position));
    }
}
