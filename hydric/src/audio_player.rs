use chrono::{DateTime, Utc};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use log::error;
use mesic::graph::{AmpNode, RenderGraph};
use shared::model::Project;
use shared::types::Volume;
use wasm_thread::JoinHandle;

// Total size of the audio buffer.
const BUFFER_SIZE: usize = 5000;

// Number of samples to render at a time.
const CHUNK_SIZE: usize = 1000;

// Don't start playing until this many samples have been produced.
const BUFFER_THRESHOLD: usize = 1000;

// For now, just samples. In future, consider supporting bars:beats, mins:secs, etc.
pub struct PlaybackPosition {
    samples: usize,
}

pub enum PlaybackMessage {
    SetProject(Box<Project>, Volume), // uses Box to keep enum size sane.
    SetAudio(Vec<Stereo<f32>>, Volume),
    Seek(PlaybackPosition),
    State(PlaybackState),
}

#[derive(PartialEq, Clone)]
pub enum PlaybackState {
    Play,
    Pause,
    Stop,
}

pub struct AudioPlayer {
    audio_tx: Option<Sender<Stereo<f32>>>,
    audio_rx: Option<Receiver<Stereo<f32>>>,
    playback_tx: Option<Sender<PlaybackMessage>>,
    playback_rx: Option<Receiver<PlaybackMessage>>,

    // TODO: make sure the producer thread is shut down when a new one starts.
    // Is losing the reference to it enough?
    stream: Option<Stream>,
    producer_thread: Option<JoinHandle<()>>,
    pub start_timestamp: Option<DateTime<Utc>>,
    state: PlaybackState,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        AudioPlayer {
            audio_tx: None,
            audio_rx: None,
            playback_tx: None,
            playback_rx: None,
            stream: None,
            producer_thread: None,
            start_timestamp: None,
            state: PlaybackState::Stop,
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

    pub fn set_project(&self, project: Box<Project>, volume: Volume) {
        self.send(PlaybackMessage::SetProject(project, volume));
    }

    pub fn set_audio(&self, audio: Vec<Stereo<f32>>, volume: Volume) {
        self.send(PlaybackMessage::SetAudio(audio, volume));
    }

    pub fn init(&mut self) {
        self.init_channels();
        self.init_producer();
        self.init_stream();
    }

    pub fn init_channels(&mut self) {
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(BUFFER_SIZE);
        let (playback_tx, playback_rx) = crossbeam_channel::unbounded();

        self.audio_tx = Some(audio_tx);
        self.audio_rx = Some(audio_rx);
        self.playback_tx = Some(playback_tx);
        self.playback_rx = Some(playback_rx);
    }

    pub fn init_producer(&mut self) {
        log::info!(
            "Available parallelism: {:?}",
            wasm_thread::available_parallelism()
        );

        let playback_rx = self.playback_rx.as_ref().unwrap().clone();
        let audio_tx = self.audio_tx.as_ref().unwrap().clone();

        self.producer_thread = Some(wasm_thread::spawn(move || {
            let mut graph = RenderGraph::default();
            log::info!("Started producer_thread {:?}", wasm_thread::current().id());
            let mut state = PlaybackState::Pause;

            // TODO: create a struct that stores the state and an impl for it.
            while state != PlaybackState::Stop {
                if let Ok(message) = playback_rx.try_recv() {
                    match message {
                        PlaybackMessage::SetProject(project, volume) => {
                            graph = RenderGraph::default();
                            graph.set_from_project(&project);
                            graph.add_output_node(AmpNode {
                                volume,
                                should_clip: true,
                            });
                        }
                        PlaybackMessage::SetAudio(audio, volume) => {
                            graph = RenderGraph::from_vec(audio);
                            graph.add_output_node(AmpNode {
                                volume,
                                should_clip: true,
                            });
                        }
                        PlaybackMessage::Seek(PlaybackPosition { samples }) => {
                            log::info!("Seeking to {}", samples);
                            graph.seek(samples);
                        }
                        PlaybackMessage::State(new_state) => {
                            state = new_state;
                        }
                    }
                }

                if state == PlaybackState::Play && audio_tx.len() < BUFFER_SIZE - CHUNK_SIZE {
                    let mut did_send = false;
                    for i in 0..CHUNK_SIZE {
                        if let Some(next) = graph.next() {
                            audio_tx.try_send(next).unwrap();
                            did_send = true;
                        } else {
                            // No more audio, so pause.
                            // Consider stopping as well, but we'll need to re-create the thread if
                            // we do this.
                            state = PlaybackState::Pause;
                            log::info!(
                                "Ran out of audio, so paused after {} samples in chunk. {:?}",
                                i,
                                wasm_thread::current().id()
                            );
                            break;
                        }
                    }
                    if did_send {
                        log::info!("Sent 1000 samples. Len: {}", audio_tx.len());
                    }
                } else {
                    sleep_ms(10);
                }
            }
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
        self.send(PlaybackMessage::State(self.state.clone()));
        self.stream
            .as_mut()
            .expect("Call .init() first!")
            .play()
            .unwrap();
        self.start_timestamp = Some(chrono::offset::Utc::now());
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Pause;
        self.send(PlaybackMessage::State(self.state.clone()));
        self.stream
            .as_mut()
            .expect("Call .init() first!")
            .pause()
            .unwrap();
        self.start_timestamp = None;
    }

    pub fn seek(&mut self, samples: usize) {
        self.send(PlaybackMessage::Seek(PlaybackPosition { samples }));
    }
}

fn sleep_ms(ms: u32) {
    log::info!("Sleeping {} ms", ms);
    let secs = 0;
    let nanos = ms * 1000 * 1000;
    wasm_thread::sleep(std::time::Duration::new(secs, nanos));
}
