use wasm_bindgen::prelude::*;
use web_sys::{AudioContext};

pub struct AudioPlayer {
    ctx: AudioContext,
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.ctx.close();
    }
}

impl AudioPlayer {
    pub fn new(audio: &Vec<f32>) -> Result<AudioPlayer, JsValue> {
        let sample_rate = 44_100;
        let seconds = 10;
        let channels = 1;

        let ctx = AudioContext::new()?;

        let buffer = ctx.create_buffer(channels, seconds * sample_rate, sample_rate as f32)?;
        // TODO: support dual channel
        buffer.copy_to_channel(audio.as_slice(), 0)?;

        let source = ctx.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&ctx.destination())?;

        // Start playing!
        source.start()?;

        Ok(AudioPlayer {
            ctx,
        })
    }
}
