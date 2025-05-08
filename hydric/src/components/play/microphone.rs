use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, MediaRecorder, MediaStream, MediaStreamConstraints, MediaRecorderOptions, Blob, BlobEvent};
use js_sys::Array;

// This tutorial was used for the general code structure: https://web.dev/articles/media-recording-audio
#[wasm_bindgen]pub struct Microphone {
    media_recorder: Option<MediaRecorder>,
    audio_chunks: Array,
    #[wasm_bindgen(skip)]
    recording_status: bool,
}


#[wasm_bindgen]
impl Microphone {
    #[wasm_bindgen(constructor)]
    pub fn new () -> Self {
        Self {
            media_recorder: None,
            audio_chunks: Array::new(),
            recording_status: false,
        }
    }

    async fn inner_start(&mut self)-> Result<(), JsValue>{

        // Clear previous data.
        self.audio_chunks = Array::new();

        // Setup js element to access media devices.
        let window = window().expect("No window found.");
        let media_devices = window.navigator().media_devices().expect("Media_devices is not supported.");
        
        /*
        We can request different things for media devices to capture, as such we need a constraint
        To specify audio only.
        */
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::from(true));

        // Get media devices.
        let promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = JsFuture::from(promise).await?;
        let stream = MediaStream::from(stream); //convert to MediaStream object

        // I think there is good browser support for wav? If not we can use webm.
        let options = MediaRecorderOptions::new();
        options.set_mime_type("audio/wav");

        let media_recorder = MediaRecorder::new_with_media_stream_and_media_recorder_options(&stream, &options)?;

        // We now add listener to continuously grab audio from mic.
        let audio_chunks = self.audio_chunks.clone();
        let on_data_available = Closure::wrap(Box::new(move |e: BlobEvent| {
            // Only store if there is actually data.
            let data = e.data().expect("Should be fine so long as we have a valid blob.");
            if data.size() > 0.0 {
                audio_chunks.push(&data);
            }
        })as Box<dyn FnMut(_)>);

        media_recorder.set_ondataavailable(Some(on_data_available.as_ref().unchecked_ref()));
        on_data_available.forget();

        media_recorder.start()?;
        self.media_recorder = Some(media_recorder);
        self.recording_status = true;

        Ok(())
    }

    async fn inner_stop(&mut self) -> Result<Vec<u8>, JsValue> {
        // Only can run if we are actively recording.
        if let Some(recorder) = &self.media_recorder {
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
            self.media_recorder = None;
            self.recording_status = false;
            
            Ok(bytes)
        } else {
            Err(JsValue::from_str("No active recording."))
        }
    }

    /*
    Wrappers to avoid non thread safe JSValue, also I didnt want to mess with your Promise::spawn.
    Looks a bit ugly tho, mb better solution available.
    */
    pub async fn stop(&mut self) -> Result<Vec<u8>, String> {
        let result = self.inner_stop().await;
        result.map_err(|e| e.as_string().unwrap_or("Unknown error".into()))
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let result = self.inner_start().await;
        result.map_err(|e| e.as_string().unwrap_or("Unknown error".into()))
    }

    pub fn is_recording(&self) -> bool {
        self.recording_status
    }

}
