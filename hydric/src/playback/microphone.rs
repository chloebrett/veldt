use crossbeam_channel::{Receiver, Sender};
use futures::FutureExt;
use js_sys::Array;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    AudioBuffer, AudioBufferSourceNode, AudioContext, AudioContextState, Blob, BlobEvent,
    MediaRecorder, MediaRecorderOptions, MediaStream, MediaStreamConstraints, window,
};

// This tutorial was used for the general code structure: https://web.dev/articles/media-recording-audio
pub struct Microphone {
    stream: Rc<RefCell<Option<MediaStream>>>,
    /// media_recorder: Records audio from microphone, constructed from stream.
    media_recorder: Option<MediaRecorder>,
    /// audio_chunks: Holds output of media_recorder.
    audio_chunks: Vec<Blob>,
    /// intermediate_data: Holds processed vec<u8> data created in convert_audio(). Can be a Rc<RefCell>>.
    intermediate_data: Arc<Mutex<Vec<u8>>>,
    /// recording_status: State variable used in MicrophoneView to manage user input.
    recording_status: bool,
    /// tx/rx: Use a stream to put media_recorder data in audio_chunks as they appear.
    tx: Sender<Blob>,
    rx: Receiver<Blob>,
    /// audio_ctx: Framework to play audio.
    audio_ctx: Rc<RefCell<Option<AudioContext>>>,
    /// curr_source: Used to play audio.
    curr_source: Rc<RefCell<Option<AudioBufferSourceNode>>>,
    /// playing_status: Arc Mutex or Rc<RefCell>> required because this can be modified at any time by AudioBufferSourceNode when it finishes playing audio.
    playing_status: Arc<Mutex<bool>>,
}

impl Microphone {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        Self {
            stream: Rc::new(RefCell::new(None)),
            media_recorder: None,
            audio_chunks: vec![],
            intermediate_data: Arc::new(Mutex::new(vec![])),
            recording_status: false,
            tx,
            rx,
            audio_ctx: Rc::new(RefCell::new(None)),
            curr_source: Rc::new(RefCell::new(None)),
            playing_status: Arc::new(Mutex::new(false)),
        }
    }

    pub fn has_permissions(&self) -> bool {
        self.stream.borrow().is_some()
    }

    pub fn update_mic_recording(&mut self) {
        while let Ok(blob) = self.rx.try_recv() {
            self.audio_chunks.push(blob);
        }
    }

    pub fn get_permissions(&mut self) -> Result<(), JsValue> {
        // Setup js element to access media devices.
        let window = window().expect("No window found.");
        let media_devices = window
            .navigator()
            .media_devices()
            .expect("Media_devices is not supported.");

        // We can request different things for media devices to capture, as such we need a constraint to specify audio only.
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&true.into());

        // Get media devices.
        let promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream_ref = Rc::clone(&self.stream);

        let future = JsFuture::from(promise).then(move |result| match result {
            Ok(stream) => {
                // Place our created MediaStream in Microphone's stream attribute.
                let stream = MediaStream::from(stream);
                *stream_ref.borrow_mut() = Some(stream);

                futures::future::ready(())
            }
            Err(_e) => futures::future::ready(()),
        });

        spawn_local(future);

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        if self.is_recording() || self.has_recording() {
            Err("Can not start recording if already recording, or if a recording already exists.")?;
        }

        // Currently using webm for browser compatibility. Note, produces an mka file.
        let options = MediaRecorderOptions::new();
        options.set_mime_type("audio/webm;codecs=opus");

        // We now add listener to continuously grab audio from mic.
        let tx = self.tx.clone();
        let on_data_available = Closure::wrap(Box::new(move |e: BlobEvent| {
            // Only store if there is actually data.
            let data = e
                .data()
                .expect("MediaRecorder should construct valid blobs.");
            if data.size() > 0.0 {
                tx.try_send(data).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        // Get a reference here as we want stream to persist after clearing mic recording.
        let stream_guard = self.stream.borrow();
        let stream = stream_guard.as_ref().expect("Stream should exist");

        let media_recorder =
            MediaRecorder::new_with_media_stream_and_media_recorder_options(stream, &options)
                .unwrap();

        let callback = on_data_available.as_ref().dyn_ref();
        media_recorder.set_ondataavailable(callback);

        // Don't drop the closure.
        // TODO: make this not be a memory leak. Store a refrence to the closure on the mic object?
        on_data_available.forget();

        let err_fn = Closure::wrap(Box::new(move |err: JsValue| {
            log::error!("an error occurred on mic stream: {err:?}")
        }) as Box<dyn FnMut(_)>);
        let err_fn = err_fn.as_ref().dyn_ref();
        media_recorder.set_onerror(err_fn);

        let callback_interval = 100;
        media_recorder
            .start_with_time_slice(callback_interval)
            .unwrap();
        self.media_recorder = Some(media_recorder);
        self.recording_status = true;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.media_recorder.as_ref().unwrap().stop().unwrap();
    }

    pub fn _convert_audio(&mut self) -> Result<(), JsValue> {
        let array = Array::new();
        for chunk in &self.audio_chunks {
            array.push(chunk);
        }

        // Convert recorded chunks to bytes.
        let blob = Blob::new_with_blob_sequence(&array)?;

        let array_buffer_promise: js_sys::Promise = blob.array_buffer();
        let future = JsFuture::from(array_buffer_promise);
        let intermediate = Arc::clone(&self.intermediate_data);
        let array_buffer_future = future.then(move |response| {
            let mut intermediate_ref = intermediate.lock().unwrap();

            // Convert to Vec<u8> for UploadSample method.
            let js_array = js_sys::Uint8Array::new(&response.unwrap());
            let mut bytes = vec![0; js_array.length() as usize];
            js_array.copy_to(&mut bytes);

            *intermediate_ref = bytes;
            futures::future::ready(())
        });

        spawn_local(array_buffer_future);
        Ok(())
    }

    pub fn play_mic_audio(&self) -> Result<(), JsValue> {
        // We dont want this code to run if we are already playing audio or if we do not have a recording.
        if *self.playing_status.lock().unwrap() || !self.has_recording() {
            return Err("Already playing or no mic recording to play.")?;
        }

        // Handling in the case we want to resume not play from start.
        if let Some(ctx) = self.audio_ctx.borrow().as_ref()
            && ctx.state() == AudioContextState::Suspended
        {
            let _ = ctx.resume()?;
            *self.playing_status.lock().unwrap() = true;

            return Ok(());
        }

        // Create our audio context.
        let audio_ctx = AudioContext::new()?;
        *self.audio_ctx.borrow_mut() = Some(audio_ctx.clone());

        // Combine our chunks.
        let array = Array::new();
        for chunk in &self.audio_chunks {
            array.push(chunk);
        }

        let blob = Blob::new_with_blob_sequence(&array)?;

        let source_clone = self.curr_source.clone();
        let playing_status_clone = self.playing_status.clone();

        spawn_local(async move {
            // Extract an array buffer, need to use map as JsFuture returns a JsValue which isn't useful.
            let array_promise = blob.array_buffer();
            let array_buffer = JsFuture::from(array_promise)
                .await
                .map(js_sys::ArrayBuffer::from)
                .unwrap();

            // Create an audio buffer.
            let decoded_promise = audio_ctx.decode_audio_data(&array_buffer).unwrap();
            let decoded = JsFuture::from(decoded_promise)
                .await
                .map(AudioBuffer::from)
                .unwrap();

            let mut source_ref = source_clone.borrow_mut();
            let source = AudioBufferSourceNode::new(&audio_ctx).unwrap();
            source.set_buffer(Some(&decoded));

            // IDK about this unwrap call, could be an issue if user has zero audio output
            // (mb skill issue tho if u making music without speakers).
            source
                .connect_with_audio_node(&audio_ctx.destination())
                .unwrap();

            // Need to handle finishing playing the audio.
            let playing_status_closure_clone = playing_status_clone.clone();
            let onended_closure = Closure::wrap(Box::new(move || {
                *playing_status_closure_clone.lock().unwrap() = false;
            }) as Box<dyn FnMut()>);

            // Marked as deprecated in the source code (go to definition). However, not in the web sys docs:
            // https://docs.rs/web-sys/latest/web_sys/struct.AudioScheduledSourceNode.html#method.set_onended
            // or the MDN docs: https://developer.mozilla.org/en-US/docs/Web/API/AudioScheduledSourceNode/ended_event
            // May be a mistake.
            #[allow(deprecated)]
            source.set_onended(Some(onended_closure.as_ref().unchecked_ref()));
            // TODO: make this not be a memory leak.
            onended_closure.forget(); // Store permanently (or manage cleanup).

            source.start().unwrap();

            *source_ref = Some(source);
            *playing_status_clone.lock().unwrap() = true;
        });
        Ok(())
    }

    pub fn pause_mic_audio(&self) -> Result<(), JsValue> {
        if !self.is_playing() {
            return Err("Can not pause if not playing.")?;
        }

        let mut playing_status_clone = self.playing_status.lock().unwrap();
        if *playing_status_clone && let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            let _ = ctx.suspend()?;
            *playing_status_clone = false;
        }
        Ok(())
    }

    // Known bug here, cant stop while paused. Unsure how to fix currently.
    pub fn stop_mic_audio(&self) -> Result<(), JsValue> {
        if !self.is_playing() {
            return Err("Can not stop if not playing.")?;
        }

        if let Some(source) = self.curr_source.borrow_mut().take() {
            // Marked as deprecated in the source code (go to definition). However, not in the web sys docs:
            // https://docs.rs/web-sys/latest/web_sys/struct.AudioScheduledSourceNode.html#method.stop
            // or the MDN docs: https://developer.mozilla.org/en-US/docs/Web/API/AudioScheduledSourceNode/stop
            // May be a mistake.
            #[allow(deprecated)]
            source.stop()?; // This is marked as depreceated, yet I can't find an alternative.
        }
        *self.playing_status.lock().unwrap() = false;
        Ok(())
    }

    pub fn clear_mic(&mut self) -> Result<(), JsValue> {
        if !self.has_recording() {
            Err("Can not clear if no mic recording is present.")?;
        }

        if let Some(ctx) = self.audio_ctx.borrow_mut().take() {
            // AudioBufferSourceNode is dropped if we stop playing, so have to check if it exists.
            if *self.playing_status.lock().unwrap() {
                // Same situation as the stop_mic_audio(). Potentially mistakenly marked as deprecated.
                #[allow(deprecated)]
                let _ = self.curr_source.borrow_mut().take().unwrap().stop();
            }
            let _ = ctx.close();
        }

        if let Some(recorder) = self.media_recorder.take() {
            recorder.stop()?;
        }

        self.audio_chunks.clear();
        *self.intermediate_data.lock().unwrap() = vec![];

        self.recording_status = false;
        *self.playing_status.lock().unwrap() = false;

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.recording_status
    }

    pub fn has_recording(&self) -> bool {
        !self.audio_chunks.is_empty()
    }

    pub fn has_converted_recording(&self) -> bool {
        let intermediate = Arc::clone(&self.intermediate_data);
        !(*intermediate.lock().unwrap().clone()).is_empty()
    }

    pub fn is_playing(&self) -> bool {
        *self.playing_status.lock().unwrap()
    }

    pub fn get_sample_bytes(&mut self) -> Vec<u8> {
        let intermediate = Arc::clone(&self.intermediate_data);
        (*intermediate.lock().unwrap().clone()).to_vec()
    }
}
