use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints, window};

pub struct Microphone {
    audio_bytes: Vec<f32>,
    recording_status: bool,
}

#[wasm_bindgen]
pub async fn request_microphone_access() -> Result<MediaStream, JsValue>{
    // Setup js element to access media devices
    let window = window().expect("No window found.");
    let media_devices = window.navigator().media_devices().expect("Media_devices is not supported.");

    /*
    We can request different things from media devices, as such we need a constraint
    To specify audio only.
    */
    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::from(true));

    // This creates request access popup.
    let media_promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .expect("Failed to get Device");

    // Convert js promise to rust future.
    JsFuture::from(media_promise)
        .await
        .map(|stream| stream.into())
}


impl Microphone {
    pub fn new () -> Self {
        Self {
            audio_bytes: Vec::new(),
            recording_status: false,
        }
    }

    pub fn toggle_recording(&mut self){
        self.recording_status = !self.recording_status;
    }

    pub async fn start(&mut self)-> Result<MediaStream, JsValue>{
        request_microphone_access().await
    }

}