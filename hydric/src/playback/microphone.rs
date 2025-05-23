use crossbeam_channel::{Receiver, Sender};
use dasp_frame::Mono;
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
    stream: Arc<Mutex<Option<MediaStream>>>,
    media_recorder: Option<MediaRecorder>,
    audio_chunks: Vec<Blob>,
    intermediate_data: Arc<Mutex<Vec<u8>>>,
    recording_status: bool,
    recording: Option<Mono<f32>>,
    tx: Sender<Blob>,
    rx: Receiver<Blob>,
    audio_ctx: Arc<Mutex<Option<AudioContext>>>,
    curr_source: Arc<Mutex<Option<AudioBufferSourceNode>>>,
    playing_status: Arc<Mutex<bool>>,
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
            recording: None,
            tx,
            rx,
            audio_ctx: Arc::new(Mutex::new(None)),
            curr_source: Arc::new(Mutex::new(None)),
            playing_status: Arc::new(Mutex::new(false)),
        }
    }

    pub fn recording(&self) -> Option<Mono<f32>> {
        self.recording
    }

    pub fn has_permissions(&self) -> bool {
        self.stream.lock().unwrap().is_some()
    }

    pub fn update(&mut self) {
        log::info!("Tried update");
        while let Ok(blob) = self.rx.try_recv() {
            log::info!("Got update");
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

        /*
        We can request different things for media devices to capture, as such we need a constraint
        To specify audio only.
        */
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::from(true));

        // Get media devices.
        let promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream_ref = Arc::clone(&self.stream);
        log::info!("Requested permissions");

        let future = JsFuture::from(promise).then(move |result| match result {
            Ok(stream) => {
                log::info!("Received permissions");
                let stream = MediaStream::from(stream);
                let mut stream_ref = stream_ref.lock().unwrap();
                *stream_ref = Some(stream.clone());
                log::info!("Set stream: {:?}", stream.clone());

                futures::future::ready(())
            }
            Err(_e) => futures::future::ready(()),
        });

        spawn_local(future);

        Ok(())
    }

    pub fn start(&mut self) {
        // I think there is good browser support for wav? If not we can use webm.
        let options = MediaRecorderOptions::new();
        options.set_mime_type("audio/ogg");

        // We now add listener to continuously grab audio from mic.
        let tx = self.tx.clone();
        let on_data_available = Closure::wrap(Box::new(move |e: BlobEvent| {
            // Only store if there is actually data.
            let data = e
                .data()
                .expect("Should be fine so long as we have a valid blob.");
            log::info!("Got data");
            if data.size() > 0.0 {
                log::info!("Got data > 0");
                tx.try_send(data).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        let stream = self
            .stream
            .lock()
            .unwrap()
            .take()
            .expect("Expected get_permissions() to have succeeded.");
        let media_recorder =
            MediaRecorder::new_with_media_stream_and_media_recorder_options(&stream, &options)
                .unwrap();

        let callback = on_data_available.as_ref().dyn_ref();
        media_recorder.set_ondataavailable(callback);

        // Don't drop the closure.
        // TODO: make this not be a memory leak. Store a refrence to the closure on the mic object?
        on_data_available.forget();

        let err_fn = Closure::wrap(Box::new(move |err: JsValue| {
            log::error!("an error occurred on mic stream: {:?}", err)
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

    pub fn convert_audio_1(&mut self) -> Result<(), JsValue> {
        let array = Array::new();
        for chunk in &self.audio_chunks {
            array.push(&chunk);
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
        // We dont want this code to run if already playing audio
        if *self.playing_status.lock().unwrap() {
            return Ok(());
        }

        // Handling in the case we want to resume not play from start
        if let Some(ctx) = self.audio_ctx.lock().unwrap().as_ref() {
            if ctx.state() == AudioContextState::Suspended {
                let _ = ctx.resume()?;
                *self.playing_status.lock().unwrap() = true;

                return Ok(());
            }
        }

        //Create our audio context
        let audio_ctx = AudioContext::new()?;
        *self.audio_ctx.lock().unwrap() = Some(audio_ctx.clone());

        //combine our chunks
        let array = Array::new();
        for chunk in &self.audio_chunks {
            array.push(chunk);
        }

        let blob = Blob::new_with_blob_sequence(&array)?;

        let source_clone = self.curr_source.clone();
        let playing_status_clone = self.playing_status.clone();

        spawn_local(async move {
            //extract an array buffer, need to do the map as JsFuture returns a JsValue which isnt useful to us
            let array_promise = blob.array_buffer();
            let array_buffer = JsFuture::from(array_promise)
                .await
                .map(js_sys::ArrayBuffer::from)
                .unwrap();
            //let decoded = JsFuture::from(audio_ctx.decode_audio_data(&array_buffer)).await.unwrap();

            //Create an audio buffer
            let decoded_promise = audio_ctx.decode_audio_data(&array_buffer).unwrap();
            let decoded = JsFuture::from(decoded_promise)
                .await
                .map(web_sys::AudioBuffer::from)
                .unwrap();
            let decoded = AudioBuffer::from(decoded);

            let mut source_guard = source_clone.lock().unwrap();
            let source = AudioBufferSourceNode::new(&audio_ctx).unwrap();
            source.set_buffer(Some(&decoded));

            // idk about this unwrap call, could be an issue if user has zero audio output
            // (mb skill issue tho if u making music without speakers)
            source
                .connect_with_audio_node(&audio_ctx.destination())
                .unwrap();

            // need to handle finishing playing the audio
            let playing_status_closure_clone = playing_status_clone.clone(); // clone a clone?
            let onended_closure = Closure::wrap(Box::new(move || {
                *playing_status_closure_clone.lock().unwrap() = false;
            }) as Box<dyn FnMut()>);

            // TODO: make this not be a memory leak.
            source.set_onended(Some(onended_closure.as_ref().unchecked_ref()));
            onended_closure.forget(); // Store permanently (or manage cleanup)

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

    //known bug here, cant stop while paused. Unsure how to fix currently
    pub fn stop_mic_audio(&self) -> Result<(), JsValue> {
        if let Some(source) = self.curr_source.lock().unwrap().take() {
            source.stop()?;
        }
        *self.playing_status.lock().unwrap() = false;
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.recording_status
    }
}
