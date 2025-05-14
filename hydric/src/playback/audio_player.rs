use super::{
    AudioBuffer, AudioProcessor, EMPTY_BUFFER, PlaybackMessage, PlaybackPosition, PlaybackState,
    PlaybackUpdate,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream};
use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Stereo;
use log::error;
use mesic::graph::RenderGraph;
use mesic::{SAMPLE_RATE, to_db};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use shared::model::PitchName;
use state::GeneratorSelector;
use std::sync::{Arc, Mutex};
use wasm_thread::JoinHandle;

const RECENT_AUDIO_SECONDS: f32 = 5.0;
const RECENT_AUDIO_SAMPLE_COUNT: usize = (RECENT_AUDIO_SECONDS * SAMPLE_RATE as f32) as usize;
const RMS_BUFFER_SAMPLES: usize = 64;

pub struct AudioPlayer {
    // The render graph, if we haven't given it to the processing thread yet.
    // If this is 'None', this just means it lives in the processing thread now.
    // It can't be communicated with directly if that's the case, so pass PlaybackMessages to
    // indirectly manipulate it instead.
    // This field will only be Some while we're waiting to init the processing thread.
    // We need to pass this exact graph to the thread, instead of creating a new one,
    // because it has a reference to a StoreData channel receiver.
    graph: Option<RenderGraph>,

    // For sending messages from processor -> player.
    audio_tx: Sender<AudioBuffer>,
    audio_rx: Receiver<AudioBuffer>,

    // For sending messages from UI -> processor.
    playback_tx: Sender<PlaybackMessage>,
    playback_rx: Receiver<PlaybackMessage>,

    // For sending messages from processor -> UI.
    update_tx: Sender<PlaybackUpdate>,
    update_rx: Receiver<PlaybackUpdate>,

    // For sending recently played/processed audio messages from processor -> UI, for visualising.
    // TODO: consider sending more than one sample at a time.
    // 44100 samples/sec / 60fps = approx 700 samples/frame.
    recent_tx: Sender<Stereo<f32>>,
    recent_rx: Receiver<Stereo<f32>>,

    // Ring buffer with the most recently played audio.
    recent_buf: AllocRingBuffer<Stereo<f32>>,
    // Total number of samples ever stored in recent_buf since it was created.
    // Helps to make visual rendering more consistent.
    // We can draw every nth sample, and have that correspond to the same samples each time.
    recent_buf_offset: usize,

    stream: Option<Stream>,
    processor_thread: Option<JoinHandle<()>>,

    pub state: PlaybackState,
    pub position: PlaybackPosition,
    is_looping: bool,

    // Delay from the processing end.
    // Updated by listening for events from the processing thread.
    // No mutex needed, because we're already using a channel.
    buffer_delay: usize,

    // Delay from the audio playback end.
    // Needs to be a mutex because it's written from a static JS callback (the data callback
    // for the AudioContext).
    output_delay: Arc<Mutex<usize>>,

    rms: dasp_rms::Rms<Stereo<f32>, [Stereo<f32>; RMS_BUFFER_SAMPLES]>,
}

impl AudioPlayer {
    pub fn new(graph: RenderGraph) -> Self {
        // This channel only ever contains zero or one messages. Each message contains BUFFER_SIZE samples.
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(1);

        // Other channels are used for message passing and are unbounded.
        let (playback_tx, playback_rx) = crossbeam_channel::unbounded();
        let (update_tx, update_rx) = crossbeam_channel::unbounded();
        let (recent_tx, recent_rx) = crossbeam_channel::unbounded();
        let rms_buffer = dasp_ring_buffer::Fixed::from([[0f32; 2]; RMS_BUFFER_SAMPLES]);

        Self {
            graph: Some(graph),
            audio_tx,
            audio_rx,
            playback_tx,
            playback_rx,
            update_tx,
            update_rx,
            recent_tx,
            recent_rx,
            recent_buf: AllocRingBuffer::from([[0.0; 2]; RECENT_AUDIO_SAMPLE_COUNT]),
            recent_buf_offset: 0,
            stream: None,
            processor_thread: None,
            state: PlaybackState::Pause,
            is_looping: false,
            position: PlaybackPosition { samples: 0 },
            buffer_delay: 0,
            output_delay: Arc::new(Mutex::new(0)),
            rms: dasp_rms::Rms::new(rms_buffer),
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

    // Get current position (min:sec) as a string.
    pub fn current_time(&self) -> String {
        let samples = self.effective_pos();

        let seconds_total = samples / (SAMPLE_RATE as usize);
        let minutes = seconds_total / 60;
        let seconds = seconds_total % 60;

        format!("{:02}:{:02}", minutes, seconds)
    }

    fn send(&self, message: PlaybackMessage) {
        self.playback_tx.try_send(message).unwrap();
    }

    pub fn refresh_mixer(&mut self) {
        self.maybe_init();
        self.send(PlaybackMessage::RecreateMixer);
    }

    pub fn set_audio(&mut self, audio: Vec<Stereo<f32>>) {
        self.maybe_init();
        self.send(PlaybackMessage::SetAudio(audio));
    }

    pub fn send_note_on(&mut self, generator: GeneratorSelector, pitch_name: PitchName) {
        self.maybe_init();
        self.send(PlaybackMessage::NoteOn(generator, pitch_name));
    }

    pub fn send_note_off(&mut self, generator: GeneratorSelector, pitch_name: PitchName) {
        self.maybe_init();
        self.send(PlaybackMessage::NoteOff(generator, pitch_name));
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

    pub fn recent_buf(&self) -> &AllocRingBuffer<Stereo<f32>> {
        &self.recent_buf
    }

    pub fn recent_buf_offset(&self) -> usize {
        self.recent_buf_offset
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

        while let Ok(update) = self.recent_rx.try_recv() {
            self.rms.next(update);
            self.recent_buf.push(update);
            self.recent_buf_offset += 1;
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
            self.recent_tx.clone(),
            self.is_looping,
            self.graph.take().expect("Expected a render graph!"),
        );

        // Currently no way to stop a thread once it's started.
        // Could add a "Stop"/"Terminate" state to have the thread exit of its own accord, if that
        // was needed.
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
                        // because the data callback is only every BUFFER_SIZE=2048 samples.
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

    pub fn audio_level(&self) -> [f32; 2] {
        let [left, right] = self.rms.current();
        [to_db(left), to_db(right)]
    }
}

fn to_samples(duration: std::time::Duration) -> usize {
    duration.mul_f32(SAMPLE_RATE as f32).as_secs() as usize
}
