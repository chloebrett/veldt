use super::{
    AudioProcessor, BUFFER_SIZE, BUFFER_THRESHOLD, PlaybackMessage, PlaybackPosition,
    PlaybackState, PlaybackUpdate,
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
    audio_tx: Option<Sender<Stereo<f32>>>,
    audio_rx: Option<Receiver<Stereo<f32>>>,

    // Messages sent from UI -> processor.
    playback_tx: Option<Sender<PlaybackMessage>>,
    playback_rx: Option<Receiver<PlaybackMessage>>,

    // Messages sent from processor -> UI.
    update_tx: Option<Sender<PlaybackUpdate>>,
    update_rx: Option<Receiver<PlaybackUpdate>>,

    // TODO: make sure the processor thread is shut down when a new one starts.
    // Is losing the reference to it enough?
    stream: Option<Stream>,
    processor_thread: Option<JoinHandle<()>>,
    pub state: PlaybackState,
    pub position: PlaybackPosition,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        AudioPlayer {
            audio_tx: None,
            audio_rx: None,
            playback_tx: None,
            playback_rx: None,
            update_tx: None,
            update_rx: None,
            stream: None,
            processor_thread: None,
            state: PlaybackState::Stop,
            position: PlaybackPosition { samples: 0 },
        }
    }
}

impl AudioPlayer {
    fn send(&self, message: PlaybackMessage) {
        self.playback_tx
            .as_ref()
            .expect("Call .init() first!")
            .try_send(message)
            .unwrap();
    }

    pub fn is_ready(&self) -> bool {
        self.stream.is_some()
    }

    pub fn set_project(&self, project: Box<Project>, volume: Volume) {
        self.send(PlaybackMessage::SetProject(project, volume));
    }

    pub fn set_audio(&self, audio: Vec<Stereo<f32>>, volume: Volume) {
        self.send(PlaybackMessage::SetAudio(audio, volume));
    }

    pub fn init(&mut self) {
        self.init_channels();
        self.init_processor();
        self.init_stream();
    }

    /// Checks for any pending updates from the processor thread and saves them locally.
    pub fn update(&mut self) {
        while let Ok(update) = self
            .update_rx
            .as_ref()
            .expect("Call .init() first!")
            .try_recv()
        {
            match update {
                PlaybackUpdate::Pos(pos) => {
                    self.position = pos;
                }
                PlaybackUpdate::State(state) => {
                    self.state = state;
                }
            }
        }
    }

    pub fn init_channels(&mut self) {
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(BUFFER_SIZE);
        self.audio_tx = Some(audio_tx);
        self.audio_rx = Some(audio_rx);

        let (playback_tx, playback_rx) = crossbeam_channel::unbounded();
        self.playback_tx = Some(playback_tx);
        self.playback_rx = Some(playback_rx);

        let (update_tx, update_rx) = crossbeam_channel::unbounded();
        self.update_tx = Some(update_tx);
        self.update_rx = Some(update_rx);
    }

    pub fn init_processor(&mut self) {
        log::info!(
            "Available parallelism: {:?}",
            wasm_thread::available_parallelism()
        );

        let playback_rx = self.playback_rx.as_ref().unwrap().clone();
        let update_tx = self.update_tx.as_ref().unwrap().clone();
        let audio_tx = self.audio_tx.as_ref().unwrap().clone();

        let mut processor = AudioProcessor::new(audio_tx, playback_rx, update_tx);

        self.processor_thread = Some(wasm_thread::spawn(move || {
            processor.run();
        }));
    }

    pub fn init_stream(&mut self) {
        // Tracks whether BUFFER_THRESHOLD has been reached.
        // Once this is true, it stays true.
        let mut latch = false;

        let audio_rx = self.audio_rx.as_ref().unwrap().clone();

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
                            if !latch && audio_rx.len() > BUFFER_THRESHOLD {
                                latch = true;
                            }

                            let value = if latch {
                                audio_rx.try_recv().unwrap_or([0.0; 2])
                            } else {
                                [0.0; 2]
                            };

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
        self.state = PlaybackState::Play;
        self.send(PlaybackMessage::State(self.state));
        self.stream
            .as_mut()
            .expect("Call .init() first!")
            .play()
            .unwrap();
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Pause;
        self.send(PlaybackMessage::State(self.state));
        self.stream
            .as_mut()
            .expect("Call .init() first!")
            .pause()
            .unwrap();
    }

    pub fn seek(&mut self, samples: usize) {
        self.position = PlaybackPosition { samples };
        self.send(PlaybackMessage::Seek(self.position));
    }
}
