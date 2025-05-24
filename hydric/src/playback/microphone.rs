use crossbeam_channel::{Receiver, Sender};
use futures::FutureExt;
use js_sys::Array;
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
    stream: Arc<Mutex<Option<MediaStream>>>, // Arc Mutex is required because stream is used in an async callback .then() in get_permissions().
    media_recorder: Option<MediaRecorder>, // Records audio from microphone, constructed from stream.
    audio_chunks: Vec<Blob>, // Holds output of media_recorder.
    intermediate_data: Arc<Mutex<Vec<u8>>>, // Holds processed vec<u8> data created in convert_audio().
    recording_status: bool, // State variable used in MicrophoneView to manage user input.
    tx: Sender<Blob>, // Use a stream to put media_recorder data in audio_chunks as they appear.
    rx: Receiver<Blob>,
    audio_ctx: Arc<Mutex<Option<AudioContext>>>, // Arc Mutex required since it is used in an async spawn_local in play_mic_audio(). Framework to play audio.
    curr_source: Arc<Mutex<Option<AudioBufferSourceNode>>>, // Arc Mutex required since it is used in an async spawn_local in play_mic_audio(). Used to play audio.
    playing_status: Arc<Mutex<bool>>, // Arc Mutex required because this can be modified at any time by AudioBufferSourceNode when it finishes playing audio.
}

impl Microphone {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        Self {
            stream: Arc::new(Mutex::new(None)),
            media_recorder: None,
            audio_chunks: vec![],
            intermediate_data: Arc::new(Mutex::new(vec![])),
            recording_status: false,
            tx,
            rx,
            audio_ctx: Arc::new(Mutex::new(None)), // Do we need these as arc? TODO
            curr_source: Arc::new(Mutex::new(None)),
            playing_status: Arc::new(Mutex::new(false)),
        }
    }

    pub fn has_permissions(&self) -> bool {
        self.stream.lock().unwrap().is_some()
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

        // We can request different things for media devices to capture, as such we need a constraint
        // to specify audio only.

        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&true.into());

        // Get media devices.
        let promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream_ref = Arc::clone(&self.stream);

        let future = JsFuture::from(promise).then(move |result| match result {
            Ok(stream) => {
                let stream = MediaStream::from(stream);
                let mut stream_ref = stream_ref.lock().unwrap();
                *stream_ref = Some(stream.clone());

                futures::future::ready(())
            }
            Err(_e) => futures::future::ready(()),
        });

        spawn_local(future);

        Ok(())
    }

    pub fn start(&mut self) {
        // Currently using ogg for browser compatibility, however wav would be better in future as it is lossless.
        let options = MediaRecorderOptions::new();
        options.set_mime_type("audio/ogg");

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
        let stream_guard = self.stream.lock().unwrap();
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
        // We dont want this code to run if we are already playing audio.
        if *self.playing_status.lock().unwrap() {
            return Ok(());
        }

        // Handling in the case we want to resume not play from start.
        if let Some(ctx) = self.audio_ctx.lock().unwrap().as_ref() {
            if ctx.state() == AudioContextState::Suspended {
                let _ = ctx.resume()?;
                *self.playing_status.lock().unwrap() = true;

                return Ok(());
            }
        }

        // Create our audio context.
        let audio_ctx = AudioContext::new()?;
        *self.audio_ctx.lock().unwrap() = Some(audio_ctx.clone());

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

            let mut source_guard = source_clone.lock().unwrap();
            let source = AudioBufferSourceNode::new(&audio_ctx).unwrap();
            source.set_buffer(Some(&decoded));

            // IDK about this unwrap call, could be an issue if user has zero audio output
            // (mb skill issue tho if u making music without speakers)
            source
                .connect_with_audio_node(&audio_ctx.destination())
                .unwrap();

            // Need to handle finishing playing the audio.
            let playing_status_closure_clone = playing_status_clone.clone(); // clone a clone?
            let onended_closure = Closure::wrap(Box::new(move || {
                *playing_status_closure_clone.lock().unwrap() = false;
            }) as Box<dyn FnMut()>);

            // TODO: make this not be a memory leak.
            source.set_onended(Some(onended_closure.as_ref().unchecked_ref()));
            onended_closure.forget(); // Store permanently (or manage cleanup).

            source.start().unwrap();

            *source_guard = Some(source);
            *playing_status_clone.lock().unwrap() = true;

            ()
        });
        Ok(())
    }

    pub fn pause_mic_audio(&self) -> Result<(), JsValue> {
        let mut playing_status_clone = self.playing_status.lock().unwrap();
        if *playing_status_clone {
            if let Some(ctx) = self.audio_ctx.lock().unwrap().as_ref() {
                let _ = ctx.suspend()?;
                *playing_status_clone = false;
            }
        }
        Ok(())
    }

    // Known bug here, cant stop while paused. Unsure how to fix currently.
    pub fn stop_mic_audio(&self) -> Result<(), JsValue> {
        if let Some(source) = self.curr_source.lock().unwrap().take() {
            source.stop()?; // This is marked as depreceated, yet I can't find an alternative.
        }
        *self.playing_status.lock().unwrap() = false;
        Ok(())
    }

    pub fn clear_mic(&mut self) -> Result<(), JsValue> {
        if let Some(ctx) = self.audio_ctx.lock().unwrap().take() {
            // AudioBufferSourceNode is dropped if we stop playing, so have to check if it exists.
            if *self.playing_status.lock().unwrap() {
                let _ = self.curr_source.lock().unwrap().take().unwrap().stop();
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

    pub fn is_playing(&self) -> bool {
        *self.playing_status.lock().unwrap()
    }
}
