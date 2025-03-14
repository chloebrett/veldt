use chrono::{DateTime, Utc};
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use web_sys::console;

pub struct Handle {
    _stream: Stream,
    pub start_timestamp: DateTime<Utc>,
}

pub fn play(audio: &[f32]) -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    let config: &cpal::StreamConfig = &config.into();

    let mut sample_clock: usize = 0;
    let audio = audio.to_owned();
    let mut next_sample = move || {
        sample_clock += 1;
        *audio.get(sample_clock).unwrap_or(&0.0)
    };
    // TODO: replace with egui logger
    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {}", err).into());
    let channels = config.channels as usize;

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| {
                for frame in data.chunks_mut(channels) {
                    let value = next_sample();
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    Handle {
        _stream: stream,
        start_timestamp: chrono::offset::Utc::now(),
    }
}
