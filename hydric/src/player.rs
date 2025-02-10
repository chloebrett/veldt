use wasm_bindgen::prelude::*;
use web_sys::{AudioContext};

#[wasm_bindgen]
pub struct AudioPlayer {
    ctx: AudioContext,

    buffer: web_sys::AudioBuffer,
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.ctx.close();
    }
}

#[wasm_bindgen]
impl AudioPlayer {
    pub fn new() -> Result<AudioPlayer, JsValue> {
        let sample_rate = 44_100; // TODO: match mesic to this
        let seconds = 10;
        let channels = 1;

        let ctx = AudioContext::new()?;
        let source = ctx.create_buffer_source()?;
        let buffer = ctx.create_buffer(channels, seconds * sample_rate, sample_rate as f32)?;

        // Fill the buffer with random noise
        //let mut channel = buffer.get_channel_data(0)?;
        let mesic_out: Vec<f32> = mesic::demo_floats();
        //for i in 0..buffer.length() as usize {
        //    channel[i] = 0.0;
            // TODO:  noise!
        //}
        buffer.copy_to_channel(&mesic_out.as_slice(), 0)?;

        // Connect the buffer to the AudioContext destination (aka
        // the user's speakers).
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&ctx.destination())?;

        // Start playing the buffer!
        source.start()?;

        Ok(AudioPlayer {
            ctx,
            buffer,
        })
    }
}
