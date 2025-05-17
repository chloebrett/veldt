use dasp_frame::Mono;
use futures::FutureExt;
use js_sys::Array;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    Blob, BlobEvent, MediaRecorder, MediaRecorderOptions, MediaStream, MediaStreamConstraints,
    window,
};

// This tutorial was used for the general code structure: https://web.dev/articles/media-recording-audio
pub struct Microphone {
    stream: Arc<Mutex<Option<MediaStream>>>,
    media_recorder: Arc<Mutex<Option<MediaRecorder>>>,
    audio_chunks: Array,
    recording_status: bool,
    recording: Option<Mono<f32>>,
}

impl Microphone {
    pub fn new() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
            media_recorder: Arc::new(Mutex::new(None)),
            audio_chunks: Array::new(),
            recording_status: false,
            recording: None,
        }
    }

    pub fn recording(&self) -> Option<Mono<f32>> {
        self.recording
    }

    //fn get_permissions() {
    //     //issue because needs static lifetime
    //     //instead use arc mutex, cloneable value, take ownership make changes and update initial var
    //     //wrap mutex in arc, which implements send allowing cloning
    //     do_something.then(|stream| *obj.lock().unwrap() = stream)
    //let stream = MediaStream::from(stream); //convert to MediaStream object
    //}

    pub fn has_permissions(&self) -> bool {
        self.stream.lock().unwrap().is_some()
    }

    pub fn get_permissions(&mut self) -> Result<(), JsValue> {
        // Clear previous data.
        self.audio_chunks = Array::new();

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
            Err(e) => futures::future::ready(()),
        });

        spawn_local(future);

        Ok(())
    }

    pub fn start(&mut self) {
        // I think there is good browser support for wav? If not we can use webm.
        let options = MediaRecorderOptions::new();
        options.set_mime_type("audio/wav");

        let stream = self.stream.lock().unwrap().take().expect("Expected get_permissions() to have succeeded.");
        let media_recorder =
            MediaRecorder::new_with_media_stream_and_media_recorder_options(&stream, &options);

        /*
        // We now add listener to continuously grab audio from mic.
        let audio_chunks = self.audio_chunks.clone();
        let on_data_available = Closure::wrap(Box::new(move |e: BlobEvent| {
            // Only store if there is actually data.
            let data = e
                .data()
                .expect("Should be fine so long as we have a valid blob.");
            if data.size() > 0.0 {
                audio_chunks.push(&data);
            }
        }) as Box<dyn FnMut(_)>);

        media_recorder.expect("IDK").set_ondataavailable(on_data_available.as_ref().unchecked_ref());
        on_data_available.forget();

        media_recorder.expect("REASON").start();
        *self.media_recorder.lock().unwrap() = Some(media_recorder.expect("REASON"));
        self.recording_status = true;
        */
    }

    pub fn stop(&mut self) {}

    async fn inner_stop(&mut self) {
        /*
        // Only can run if we are actively recording.
        if let Some(recorder) = *self.media_recorder.lock().unwrap() {
            recorder.stop()?;

            // Convert recorded chunks to bytes.
            let blob = Blob::new_with_blob_sequence(&self.audio_chunks)?;

            let array_buffer_promise = blob.array_buffer();
            let array_buffer = JsFuture::from(array_buffer_promise).await?;

            // Convert to Vec<u8> for UploadSample method.
            let js_array = js_sys::Uint8Array::new(&array_buffer);
            let mut bytes = vec![0; js_array.length() as usize];
            js_array.copy_to(&mut bytes);

            // Clean up.
            *self.media_recorder.lock().unwrap() = None;
            self.recording_status = false;

            Ok(bytes)
        } else {
            Err(JsValue::from_str("No active recording."))
        }
        */
    }

    pub fn is_recording(&self) -> bool {
        self.recording_status
    }
}
