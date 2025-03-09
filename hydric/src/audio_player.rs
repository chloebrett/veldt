use chrono::{DateTime, offset::Utc};
use mesic::SAMPLE_RATE;
use wasm_bindgen::prelude::*;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext};

pub struct AudioPlayer {
    ctx: AudioContext,
    source: AudioBufferSourceNode,
    buffer: AudioBuffer,

    pub playback_start_timestamp: Option<DateTime<Utc>>,
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.ctx.close();
    }
}

impl AudioPlayer {
    pub fn new() -> Result<AudioPlayer, JsValue> {
        let seconds = 10; // TODO: generalise
        let channels = 1;

        let ctx = AudioContext::new()?;

        let source = ctx.create_buffer_source()?;
        let buffer =
            ctx.create_buffer(channels, seconds * SAMPLE_RATE as u32, SAMPLE_RATE as f32)?;

        Ok(AudioPlayer {
            ctx,
            source,
            buffer,
            playback_start_timestamp: None,
        })
    }

    pub fn set_audio(&mut self, audio: &[f32]) -> Result<(), JsValue> {
        // TODO: support dual channel
        let channels = 1;
        self.buffer = self
            .ctx
            .create_buffer(channels, audio.len() as u32, SAMPLE_RATE as f32)?;
        self.buffer.copy_to_channel(audio, 0)?;

        // Note: each source can only be used once, but they are inexpensive to create.
        // See https://developer.mozilla.org/en-US/docs/Web/API/AudioBufferSourceNode.
        self.source = self.ctx.create_buffer_source()?;
        self.source.set_buffer(Some(&self.buffer));
        self.source
            .connect_with_audio_node(&self.ctx.destination())?;

        Ok(())
    }

    pub fn play(&mut self) -> Result<(), JsValue> {
        self.source.start()?;
        self.playback_start_timestamp = Some(chrono::offset::Utc::now());

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), JsValue> {
        // I think this was marked deprecated by mistake in bindgen;
        // docs say it's fine.
        // https://developer.mozilla.org/en-US/docs/Web/API/AudioScheduledSourceNode/stop
        #[allow(deprecated)]
        self.source.stop()?;
        self.playback_start_timestamp = None;

        Ok(())
    }
}
