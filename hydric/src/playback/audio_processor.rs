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
    is_looping: bool,
}

impl AudioProcessor {
    pub fn new(
        audio_tx: Sender<Stereo<f32>>,
        playback_rx: Receiver<PlaybackMessage>,
        update_tx: Sender<PlaybackUpdate>,
        is_looping: bool,
        graph: RenderGraph,
    ) -> Self {
        AudioProcessor {
            audio_tx,
            playback_rx,
            update_tx,
            is_looping,
            state: PlaybackState::Pause,
            graph,
        }
    }

    pub fn run(&mut self) {
        log::info!(
            "Started audio processor on thread {:?}",
            wasm_thread::current().id()
        );

        loop {
            log::info!("Loop");
            self.read_messages();

            if self.state == PlaybackState::Play && self.audio_tx.len() < BUFFER_SIZE - CHUNK_SIZE {
                log::info!("Process");
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
                    self.graph.clear_nodes();
                    self.graph.set_from_project(&project);
                    self.graph.add_output_amp_node(AmpNode {
                        volume,
                        should_clip: true,
                    });
                }
                PlaybackMessage::SetAudio(audio, volume) => {
                    self.graph.clear_nodes();
                    self.graph.set_from_audio(audio);
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
                PlaybackMessage::Loop(is_looping) => {
                    self.is_looping = is_looping;
                }
            }
        }
    }

    fn process_chunk(&mut self) {
        let mut did_send = false;
        log::info!("Will loop");
        for i in 0..CHUNK_SIZE {
            log::info!("Looping {:?}", self.graph);
            if let Some(next) = self.graph.next() {
                log::info!("Sending");
                self.audio_tx.try_send(next).unwrap();
                log::info!("Sent");
                did_send = true;
            } else if self.is_looping {
                log::info!(
                    "Ran out of audio after {} samples in chunk. Looping. {:?}",
                    i,
                    wasm_thread::current().id()
                );
                self.graph.seek(0);
            } else {
                log::info!("Ran out");
                // No more audio, so finish.
                self.state = PlaybackState::Finished;
                self.update_tx
                    .try_send(PlaybackUpdate::State(self.state))
                    .unwrap();
                log::info!(
                    "Ran out of audio, so marked finished after {} samples in chunk. {:?}",
                    i,
                    wasm_thread::current().id()
                );
                break;
            }
            log::info!("End loop");
        }
        log::info!("Looped");
        if did_send {
            log::info!("Did send");
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
