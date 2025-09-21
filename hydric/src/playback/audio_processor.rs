use super::{
    AudioBuffer, BUFFER_SIZE, EMPTY_BUFFER, PlaybackMessage, PlaybackPosition, PlaybackState,
    PlaybackUpdate,
};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use mesic::{consts::MAX_MIXER_CHANNELS, graph::RenderGraph};

/// Audio processor which runs in its own thread and communicates with the UI thread via crossbeam channels.
pub struct AudioProcessor {
    // Audio processor gets one half of each of the three channels.
    // It receives playback messages, and sends audio buffers and playback update messages.
    audio_tx: Sender<AudioBuffer>,
    playback_rx: Receiver<PlaybackMessage>,
    update_tx: Sender<PlaybackUpdate>,
    recent_tx: [Sender<Stereo<f32>>; MAX_MIXER_CHANNELS],
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
        recent_tx: [Sender<Stereo<f32>>; MAX_MIXER_CHANNELS],
        is_looping: bool,
        graph: RenderGraph,
    ) -> Self {
        Self {
            audio_tx,
            playback_rx,
            update_tx,
            recent_tx,
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
            // Process pending messages (non-blocking).
            while let Ok(message) = self.playback_rx.try_recv() {
                self.process_message(message);
            }

            if self.state == PlaybackState::Play && self.audio_tx.is_empty() {
                self.process_chunk();
            } else {
                // 10ms is chosen because:
                // We send 2048 samples at a time (BUFFER_SIZE).
                // This takes 2048 / 44100 seconds = 46ms to play.
                // At some point every 46ms (assuming perfect playback), the playback thread is likely to consume the
                // next buffer content. Therefore we can choose any value up to almost 46ms as the
                // sleep duration. We choose 10ms just in case things end up out of sync somehow,
                // so we still poll several times to see if the audio thread wants more data.
                // This sleep call is necessary - if we don't have one, the processing thread
                // busy-waits and consumes all the CPU, making the app unusable.
                // This could also happen if we made the sleep time too low (e.g. 100
                // microseconds).
                // Note: when we sleep, we're not listening for playback events (pause, seek etc)
                // either. But 10ms is an acceptable interactive lag, and is shorter than the
                // framerate (16.7ms) anyway.
                // TODO: consider replacing this sleep call with a mutex wait that somehow listens
                // both for "audio player is ready to receive" and also "playback_rx has a new
                // message".
                sleep_ms(10);
            }
            self.update_tx
                .try_send(PlaybackUpdate::Delay(self.audio_tx.len()))
                .unwrap();
        }
    }

    fn process_message(&mut self, message: PlaybackMessage) {
        match message {
            PlaybackMessage::RecreateMixer => {
                self.graph.recreate_mixer();
            }
            PlaybackMessage::PassDebugChannelToMixer(debug_tx) => {
                self.graph.set_debug_tx(debug_tx);
            }
            PlaybackMessage::SetAudio(audio) => {
                self.graph.set_audio(&audio);
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
            PlaybackMessage::NoteOn(generator, pitch_name) => {
                self.graph.note_on(generator, pitch_name);
            }
            PlaybackMessage::NoteOff(generator, pitch_name) => {
                self.graph.note_off(generator, pitch_name);
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
                Some(channels) => {
                    self.buffer[i] = channels[0];
                    for (index, value) in channels.iter().enumerate() {
                        self.recent_tx[index].try_send(*value).unwrap();
                    }
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
