use super::{
    AudioBuffer, BUFFER_SIZE, EMPTY_BUFFER, PlaybackMessage, PlaybackPosition, PlaybackState,
    PlaybackUpdate,
};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use mesic::graph::RenderGraph;

/// Audio processor which runs in its own thread and communicates with the UI thread via crossbeam channels.
pub struct AudioProcessor {
    audio_tx: Sender<AudioBuffer>,
    playback_rx: Receiver<PlaybackMessage>,
    update_tx: Sender<PlaybackUpdate>,
    state: PlaybackState,
    graph: RenderGraph,
    is_looping: bool,
    buffer: AudioBuffer,
}

impl AudioProcessor {
    pub fn new(
        audio_tx: Sender<AudioBuffer>,
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
            buffer: EMPTY_BUFFER,
        }
    }

    pub fn run(&mut self) {
        log::info!(
            "Started audio processor on thread {:?}",
            wasm_thread::current().id()
        );

        loop {
            self.read_messages();

            if self.state == PlaybackState::Play && self.audio_tx.is_empty() {
                self.process_chunk();
            } else {
                // TODO: consider replacing this with a blocking .recv
                // that waits for a new action if we'd otherwise be paused/finished.
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
                PlaybackMessage::SetProject(project) => {
                    self.graph.clear_nodes();
                    self.graph.set_from_project(&project);
                }
                PlaybackMessage::SetAudio(audio) => {
                    self.graph.clear_nodes();
                    self.graph.set_from_audio(audio);
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
        let mut got_samples = false;
        self.buffer.copy_from_slice(&EMPTY_BUFFER);
        for i in 0..BUFFER_SIZE {
            let mut next = self.graph.next();
            if next.is_none() && self.is_looping {
                log::info!("Ran out of audio after {i} samples in buffer. Looping.");
                self.graph.seek(0);
                next = self.graph.next();
            }

            match next {
                Some(value) => {
                    self.buffer[i] = value;
                    got_samples = true;
                }
                None => {
                    // No more audio, so finish.
                    self.state = PlaybackState::Finished;
                    self.update_tx
                        .try_send(PlaybackUpdate::State(self.state))
                        .unwrap();
                    log::info!("Ran out of audio, so marked finished after {i} samples in buffer.");
                    break;
                }
            }
        }

        if got_samples {
            self.audio_tx.try_send(self.buffer).unwrap();
            self.update_tx
                .try_send(PlaybackUpdate::Pos(PlaybackPosition {
                    samples: self.graph.pos(),
                }))
                .unwrap();
            log::info!("Sent a buffer of samples.");
        }
    }
}

fn sleep_ms(ms: u32) {
    log::info!("Sleeping {} ms", ms);
    let secs = 0;
    let nanos = ms * 1000 * 1000;
    wasm_thread::sleep(std::time::Duration::new(secs, nanos));
}
