use chrono::{DateTime, Utc};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Sender;
use dasp_frame::Stereo;
use log::error;
use mesic::graph::{AmpNode, RenderGraph};
use shared::model::Project;
use shared::types::Volume;
use wasm_thread::JoinHandle;

// For now, just samples. In future, consider supporting bars:beats, mins:secs, etc.
struct PlaybackPosition {
    samples: usize,
}

pub enum PlaybackMessage {
    SetProject(Box<Project>, Volume), // uses Box to keep enum size sane.
    SetAudio(Box<Vec<Stereo<f32>>>, Volume), // uses Box to keep enum size sane.
    Seek(PlaybackPosition),
    State(PlaybackState),
}

#[derive(PartialEq)]
enum PlaybackState {
    Play,
    Pause,
    Stop,
}

#[derive(Default)]
pub struct AudioPlayer {
    playback_tx: Option<Sender<PlaybackMessage>>,
    stream: Option<Stream>,
    producer_thread: Option<JoinHandle<()>>,
    pub start_timestamp: Option<DateTime<Utc>>,
}

impl AudioPlayer {
    pub fn reset(&mut self) {
        self.playback_tx = None;
        self.stream = None;
        self.producer_thread = None;
        self.start_timestamp = None;

        // TODO: make sure the producer thread is shut down.
    }

    pub fn send(&self, message: PlaybackMessage) {
        self.playback_tx
            .as_ref()
            .expect("Call .init() first!")
            .try_send(message)
            .unwrap();
    }

    pub fn init(&mut self) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();
        let config: &cpal::StreamConfig = &config.into();

        let err_fn = |err| error!("an error occurred on stream: {}", err);
        let channels = config.channels as usize;

        // Total size of the audio buffer.
        let buffer_size = 5000;

        // Number of samples to render at a time.
        let chunk_size = 1000;

        // Don't start playing until this many samples have been produced.
        let buffer_threshold = 1000;

        let (audio_tx, audio_rx) = crossbeam_channel::bounded(buffer_size);
        let (playback_tx, playback_rx) = crossbeam_channel::unbounded();

        self.playback_tx = Some(playback_tx);

        log::info!(
            "Available parallelism: {:?}",
            wasm_thread::available_parallelism()
        );

        let producer_thread = wasm_thread::spawn(move || {
            let mut graph = RenderGraph::default();
            log::info!("Started producer_thread {:?}", wasm_thread::current().id());
            let mut state = PlaybackState::Pause;

            while state != PlaybackState::Stop {
                if let Ok(message) = playback_rx.try_recv() {
                    match message {
                        PlaybackMessage::SetProject(project, volume) => {
                            graph = RenderGraph::default();
                            graph.set_from_project(&project);
                        }
                        PlaybackMessage::SetAudio(audio, volume) => {
                            graph = RenderGraph::from_vec(*audio);
                            graph.add_output_node(AmpNode {
                                volume,
                                should_clip: true,
                            });
                        }
                        PlaybackMessage::Seek(position) => todo!(),
                        PlaybackMessage::State(new_state) => {
                            state = new_state;
                        }
                    }
                }

                if state == PlaybackState::Play && audio_tx.len() < buffer_size - chunk_size {
                    for _ in 0..chunk_size {
                        if let Some(next) = graph.next() {
                            let _ = audio_tx.try_send(next).unwrap();
                            log::info!("Sent 1000 samples. Len: {}", audio_tx.len());
                        } else {
                            // No more audio, so pause.
                            // Consider stopping as well, but we'll need to re-create the thread if
                            // we do this.
                            state = PlaybackState::Pause;
                            log::info!(
                                "Ran out of audio, so paused. {:?}",
                                wasm_thread::current().id()
                            );
                        }
                    }
                } else {
                    sleep_ms(10);
                }
            }
        });

        // Tracks whether buffer_threshold has been reached.
        // Once this is true, it stays true.
        let mut latch = false;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _| {
                    for frame in data.chunks_mut(channels) {
                        if !latch && audio_rx.len() > buffer_threshold {
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
            .unwrap();

        self.stream = Some(stream);
        self.producer_thread = Some(producer_thread);
    }

    pub fn play(&mut self) {
        self.send(PlaybackMessage::State(PlaybackState::Play));
        self.stream
            .as_mut()
            .expect("Call .init() first!")
            .play()
            .unwrap();
        self.start_timestamp = Some(chrono::offset::Utc::now());
    }
}

fn sleep_ms(ms: u32) {
    log::info!("Sleeping {} ms", ms);
    let secs = 0;
    let nanos = ms * 1000 * 1000;
    wasm_thread::sleep(std::time::Duration::new(secs, nanos));
}
