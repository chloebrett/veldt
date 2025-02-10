use wasm_bindgen::prelude::*;
use web_sys::{AudioContext};
use shared::model::wave_type::WaveType;

pub struct AudioPlayer {
    ctx: AudioContext,

    buffer: web_sys::AudioBuffer,
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.ctx.close();
    }
}

impl AudioPlayer {
    pub fn new(wave: WaveType) -> Result<AudioPlayer, JsValue> {
        let sample_rate = 44_100;
        let seconds = 10;
        let channels = 1;

        let ctx = AudioContext::new()?;

        let buffer = ctx.create_buffer(channels, seconds * sample_rate, sample_rate as f32)?;
        let mesic_out: Vec<f32> = mesic::demo_floats(wave);
        buffer.copy_to_channel(&mesic_out.as_slice(), 0)?;

        let source = ctx.create_buffer_source()?;
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&ctx.destination())?;

        // Start playing!
        source.start()?;

        Ok(AudioPlayer {
            ctx,
            buffer,
        })
    }
}
