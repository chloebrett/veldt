use super::{
    BUFFER_SIZE, CHUNK_SIZE, PlaybackMessage, PlaybackPosition, PlaybackState, PlaybackUpdate,
};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use mesic::graph::{AmpNode, RenderGraph};

/// Audio processor which runs in its own thread and communicates with the UI thread via crossbeam channels.
pub struct AudioProcessor {
    audio_tx: Sender<Stereo<f32>>,
    playback_rx: Receiver<PlaybackMessage>,
    update_tx: Sender<PlaybackUpdate>,
    state: PlaybackState,
    graph: RenderGraph,
}

impl AudioProcessor {
    pub fn new(
        audio_tx: Sender<Stereo<f32>>,
        playback_rx: Receiver<PlaybackMessage>,
        update_tx: Sender<PlaybackUpdate>,
    ) -> Self {
        AudioProcessor {
            audio_tx,
            playback_rx,
            update_tx,
            state: PlaybackState::Pause,
            graph: RenderGraph::default(),
        }
    }

    pub fn run(&mut self) {
        log::info!(
            "Started audio processor on thread {:?}",
            wasm_thread::current().id()
        );

        loop {
            self.read_messages();

            if self.state == PlaybackState::Play && self.audio_tx.len() < BUFFER_SIZE - CHUNK_SIZE {
                self.process_chunk();
            } else {
                sleep_ms(10);
            }
            self.update_tx
                .try_send(PlaybackUpdate::Delay(self.audio_tx.len()))
                .unwrap();
        }
    }

    fn read_messages(&mut self) {
        while let Ok(message) = self.playback_rx.try_recv() {
            match message {
                PlaybackMessage::SetProject(project, volume) => {
                    self.graph = RenderGraph::default();
                    self.graph.set_from_project(&project);
                    self.graph.add_output_amp_node(AmpNode {
                        volume,
                        should_clip: true,
                    });
                }
                PlaybackMessage::SetAudio(audio, volume) => {
                    self.graph = RenderGraph::from_vec(audio);
                    self.graph.add_output_amp_node(AmpNode {
                        volume,
                        should_clip: true,
                    });
                }
                PlaybackMessage::Seek(PlaybackPosition { samples }) => {
                    log::info!("Seeking to {}", samples);
                    self.graph.seek(samples);
                }
                PlaybackMessage::State(state) => {
                    log::info!(
                        "Got playback state message {:?} on thread {:?}",
                        state,
                        wasm_thread::current().id()
                    );
                    self.state = state;
                }
            }
        }
    }

    fn process_chunk(&mut self) {
        let mut did_send = false;
        for i in 0..CHUNK_SIZE {
            if let Some(next) = self.graph.next() {
                self.audio_tx.try_send(next).unwrap();
                did_send = true;
            } else {
                // No more audio, so pause.
                self.state = PlaybackState::Pause;
                self.update_tx
                    .try_send(PlaybackUpdate::State(self.state))
                    .unwrap();
                log::info!(
                    "Ran out of audio, so paused after {} samples in chunk. {:?}",
                    i,
                    wasm_thread::current().id()
                );
                break;
            }
        }
        if did_send {
            self.update_tx
                .try_send(PlaybackUpdate::Pos(PlaybackPosition {
                    samples: self.graph.pos(),
                }))
                .unwrap();
            log::info!("Sent 1000 samples. Len: {}", self.audio_tx.len());
        }
    }
}

fn sleep_ms(ms: u32) {
    log::info!("Sleeping {} ms", ms);
    let secs = 0;
    let nanos = ms * 1000 * 1000;
    wasm_thread::sleep(std::time::Duration::new(secs, nanos));
}
